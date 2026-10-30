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
    if description.record == "TLSDESCCALL" {
        if let MachineOperand::Expression(expression) = instruction.operand(0) {
            return Ok(Encoded {
                bytes: Vec::new(),
                fixups: vec![Fixup {
                    offset: 0,
                    expression: expression.clone(),
                    kind: FixupKind::Literal(569),
                    pc_relative: false,
                }],
            });
        }
    }
    if description.record == "TLSAUTHDESCCALL" {
        return Err(unencodable(
            "TLSAUTHDESCCALL: the relocation number of the authenticated TLS descriptor call is not available",
        ));
    }
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
    pub order: u64,
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
    pub range_prefetches: BTreeMap<String, SystemEntry>,
    pub range_prefetch_names: BTreeMap<u32, String>,
    pub pstate_one: BTreeMap<String, SystemEntry>,
    pub pstate_fifteen: BTreeMap<String, SystemEntry>,
    pub pstate_one_encodings: BTreeSet<u32>,
    pub pstate_fifteen_encodings: BTreeSet<u32>,
    pub hints: BTreeMap<String, SystemEntry>,
    pub exact_floats: BTreeMap<u32, String>,
    pub sys_aliases: BTreeMap<&'static str, Vec<SysAlias>>,
}

#[derive(Clone, Debug)]
pub struct SysAlias {
    pub order: u64,
    pub name: String,
    pub encoding: u32,
    pub requires: Vec<String>,
    pub needs_register: bool,
    pub optional_register: bool,
    pub allow_tlbid: bool,
}

fn system_entry(record: &TableRecord) -> SystemEntry {
    SystemEntry {
        order: record.definition_order,
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
                } else if record.is_a("RPRFM") {
                    let held = entry();
                    tables
                        .range_prefetch_names
                        .insert(held.encoding, held.name.clone());
                    tables.range_prefetches.insert(lower(&held), held);
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
                } else if let Some(group) = [
                    "IC",
                    "DC",
                    "AT",
                    "TLBIEntry",
                    "TLBIPEntry",
                    "GIC",
                    "GSB",
                    "GICR",
                    "PLBIEntry",
                ]
                .into_iter()
                .find(|class| record.is_a(class))
                {
                    let requires_text = if record.field("RequiresStr").is_some() {
                        table_text(record, "RequiresStr")
                    } else {
                        table_text(record, "Requires")
                    };
                    let reg_use = match record.field("RegUse") {
                        Some(TableValue::Record(name)) => name.clone(),
                        _ => String::new(),
                    };
                    let needs_register = match group {
                        "IC" | "GIC" | "GICR" => table_bit(record, "NeedsReg"),
                        "TLBIEntry" | "PLBIEntry" => reg_use == "REG_REQUIRED",
                        "GSB" => false,
                        _ => true,
                    };
                    tables
                        .sys_aliases
                        .entry(match group {
                            "TLBIEntry" => "TLBI",
                            "TLBIPEntry" => "TLBIP",
                            "PLBIEntry" => "PLBI",
                            "IC" => "IC",
                            "DC" => "DC",
                            "GIC" => "GIC",
                            "GSB" => "GSB",
                            "GICR" => "GICR",
                            _ => "AT",
                        })
                        .or_default()
                        .push(SysAlias {
                            order: record.definition_order,
                            name: table_text(record, "Name"),
                            encoding: table_integer(record, "Encoding") as u32,
                            requires: requirement_features(&requires_text),
                            needs_register,
                            optional_register: reg_use == "REG_OPTIONAL",
                            allow_tlbid: table_bit(record, "AllowWithTLBID"),
                        });
                } else if record.is_a("ExactFPImm") {
                    tables.exact_floats.insert(
                        table_integer(record, "Enum") as u32,
                        table_text(record, "Repr"),
                    );
                }
            }
        }
        if tables.hints.is_empty() {
            tables.hints.insert(
                "nb".to_string(),
                SystemEntry {
                    order: 0,
                    name: "nb".to_string(),
                    encoding: 1,
                    readable: true,
                    writeable: true,
                    requires: Vec::new(),
                },
            );
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
            set_feature(target, &mut features, &feature);
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
            let held = record.name.clone();
            if enable {
                set_feature(target, &mut features, &held);
            } else {
                clear_feature(target, &mut features, &held);
            }
        }
    }
    features
}

pub fn unknown_features(target: &Target, attributes: &[String]) -> Vec<String> {
    let mut unknown = Vec::new();
    for attribute in attributes {
        let (sign, name) = match attribute.strip_prefix('-') {
            Some(name) => ('-', name),
            None => ('+', attribute.trim_start_matches('+')),
        };
        let known = name == "all"
            || target.records.values().any(|record| {
                record.is_a("SubtargetFeature") && table_text(record, "Name") == name
            });
        if !known {
            unknown.push(format!("{sign}{name}"));
        }
    }
    unknown
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
            Diag::Near
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
            if let Expression::Constant(value) = right.as_ref() {
                let (specifier, offset) = classify_symbol(left)?;
                return Some((specifier, offset + sign * value));
            }
            if let (1, Expression::Constant(value)) = (sign, left.as_ref()) {
                let (specifier, offset) = classify_symbol(right)?;
                return Some((specifier, offset + value));
            }
            None
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
        if self.in_register_class("GPR64all", register) {
            return register;
        }
        let record = self.registers[register as usize].record.as_str();
        match record {
            "WZR" => return self.named_register("XZR"),
            "WSP" => return self.named_register("SP"),
            "W29" => return self.named_register("FP"),
            "W30" => return self.named_register("LR"),
            _ => {}
        }
        let encoding = self.registers[register as usize].encoding;
        self.named_register(&format!("X{encoding}"))
    }

    fn arm_w_from_x(&self, register: RegisterId) -> RegisterId {
        if self.in_register_class("GPR32all", register) {
            return register;
        }
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
                _ => return Diag::Near,
            };
            if let Some(shift) = shift {
                if shift != 0 && shift != 12 {
                    return Diag::Near;
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
                return Diag::Near;
            }
            match shifted_value(operand, 12) {
                Some((value, _)) => Diag::of(value < 0 && -value <= 0xfff),
                None => Diag::Near,
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
                return Diag::Near;
            };
            let bits = bytes * 8;
            let upper: u64 = if bits >= 64 { 0 } else { u64::MAX << bits };
            let value = value as u64;
            if value & upper != 0 && value & upper != upper {
                return Diag::Near;
            }
            Diag::of(is_logical_immediate(value & !upper, bits as u32))
        }
        "isSIMDImmType10" => Diag::of(operand.constant().is_some_and(is_simd_type10)),
        "isBranchTarget" => {
            let bits = argument_number(&arguments, 0);
            let Some(expression) = operand.immediate() else {
                return Diag::Near;
            };
            match expression.constant_value() {
                None => Diag::Match,
                Some(value) => {
                    if value & 3 != 0 {
                        return Diag::Near;
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
                return Diag::Near;
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
                return Diag::Near;
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
                return Diag::Near;
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
                return Diag::Near;
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
        ArmOperand::ShiftExtend(shift)
            if shift.amount <= 4
                && matches!(
                    shift.kind,
                    ShiftExtendKind::Uxtb
                        | ShiftExtendKind::Sxtb
                        | ShiftExtendKind::Uxth
                        | ShiftExtendKind::Sxth
                        | ShiftExtendKind::Uxtw
                        | ShiftExtendKind::Sxtw
                        | ShiftExtendKind::Uxtx
                        | ShiftExtendKind::Sxtx
                        | ShiftExtendKind::Lsl
                ) =>
        {
            Some(*shift)
        }
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
        if self.arm_keep_symbols {
            return Ok(self.arm_fold_constants(expression));
        }
        if !matches!(expression, Expression::Constant(_)) {
            if let Some(value) = self.context.evaluate_absolute(&expression, None) {
                return Ok(Expression::Constant(value));
            }
        }
        Ok(expression)
    }

    fn arm_fold_constants(&mut self, expression: Expression) -> Expression {
        match expression {
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.arm_fold_constants(*left);
                let right = self.arm_fold_constants(*right);
                let combined = Expression::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                };
                match &combined {
                    Expression::Binary { left, right, .. }
                        if matches!(**left, Expression::Constant(_))
                            && matches!(**right, Expression::Constant(_)) =>
                    {
                        match self.context.evaluate_absolute(&combined, None) {
                            Some(value) => Expression::Constant(value),
                            None => combined,
                        }
                    }
                    _ => combined,
                }
            }
            other => other,
        }
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
        let kind = name[head_end..].to_ascii_lowercase();
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
        let specifier_location = self.location();
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
        self.arm_keep_symbols = specifier.is_some();
        let parsed = self.arm_expression();
        self.arm_keep_symbols = false;
        let expression = parsed?;
        match specifier {
            None => Ok(expression),
            Some(specifier) => match self.apply_specifier(&expression, &specifier) {
                Some(applied) => {
                    self.expression_locations
                        .push((applied.clone(), Some(specifier_location)));
                    Ok(applied)
                }
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
        self.arm_try_parse_prefetch_kind(operands, false)
    }

    fn arm_try_parse_prefetch_kind(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        range: bool,
    ) -> Status {
        let tables = system_tables_for();
        let (names, by_name, maximum) = if range {
            (&tables.range_prefetch_names, &tables.range_prefetches, 63)
        } else {
            (&tables.prefetch_names, &tables.prefetches, 31)
        };
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
            if !(0..=maximum).contains(&value) {
                self.token_error(&format!(
                    "prefetch operand out of range, [0,{maximum}] expected"
                ));
                return Status::Failure;
            }
            let name = names.get(&(value as u32)).cloned().unwrap_or_default();
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
        let Some(entry) = by_name.get(&text.to_ascii_lowercase()) else {
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
            self.error(location, "expected register");
            return Status::Failure;
        }
        let Some(first) = self.arm_try_parse_scalar_register() else {
            self.error(
                location,
                "expected first even register of a consecutive same-size even/odd register pair",
            );
            return Status::Failure;
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
        if first_encoding % 2 != 0 {
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
        let part = |number: u32| -> String {
            match (wide, number) {
                (true, 29) => "FP".to_string(),
                (true, 30) => "LR".to_string(),
                (true, 31) => "XZR".to_string(),
                (true, _) => format!("X{number}"),
                (false, 31) => "WZR".to_string(),
                (false, _) => format!("W{number}"),
            }
        };
        let pair = format!("{}_{}", part(first_encoding), part(second_encoding));
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
        let name = (0..8)
            .map(|offset| match encoding + offset {
                29 => "FP".to_string(),
                number => format!("X{number}"),
            })
            .collect::<Vec<String>>()
            .join("_");
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
            "tryParseRPRFMOperand" => self.arm_try_parse_prefetch_kind(operands, true),
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
            TokenKind::Equal => self.arm_parse_literal_load(operands),
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
        if matches!(
            head.as_str(),
            "ic" | "dc" | "at" | "tlbi" | "cfp" | "dvp" | "cpp" | "cosp" | "gic" | "gsb" | "plbi"
        ) {
            if name.contains('.') {
                self.token_error("invalid operand");
                return Err(());
            }
            return self.arm_parse_sys_alias(&head, name_start);
        }
        if head == "gicr" {
            return self.arm_parse_sysl_alias(&head, name_start);
        }
        if head == "tlbip" {
            if name.contains('.') {
                self.token_error("invalid operand");
                return Err(());
            }
            return self.arm_parse_sysp_alias(&head, name_start);
        }
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
            self.arm_suffixes.insert(suffix_start);
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
            self.arm_suffixes.insert(name_start + start + 1);
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

pub fn requirement_text(target: &Target, value: &TableValue) -> (String, bool) {
    match value {
        TableValue::Record(name) => (
            target
                .records
                .get(name)
                .map(|record| table_text(record, "Name"))
                .unwrap_or_else(|| name.clone()),
            false,
        ),
        TableValue::Dag {
            operator,
            arguments,
            ..
        } => {
            let TableValue::Record(operator) = operator.as_ref() else {
                return (String::new(), false);
            };
            let parts: Vec<(String, bool)> = arguments
                .iter()
                .filter(|(argument, _)| {
                    !matches!(argument, TableValue::Record(name) if name == "FeatureAll")
                })
                .map(|(argument, _)| requirement_text(target, argument))
                .collect();
            if operator == "not" {
                let inner = parts.first().cloned().unwrap_or_default();
                return (format!("!{}", inner.0), true);
            }
            if parts.len() == 1 {
                return parts[0].clone();
            }
            let joiner = if operator == "any_of" {
                " or "
            } else {
                " and "
            };
            let rendered: Vec<String> = parts
                .iter()
                .map(|(text, compound)| {
                    if *compound {
                        format!("({text})")
                    } else {
                        text.clone()
                    }
                })
                .collect();
            (rendered.join(joiner), true)
        }
        _ => (String::new(), false),
    }
}

fn predicates_features(requires: &[String], features: &BTreeSet<String>) -> bool {
    features.contains("FeatureAll")
        || requires
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
    if op0 > 3 || op1 > 7 || crn > 15 || crm > 15 || op2 > 7 {
        return None;
    }
    Some((op0 << 14) | (op1 << 11) | (crn << 7) | (crm << 3) | op2)
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
                        candidate.kind == ClassKind::Token
                            && candidate.value_name.eq_ignore_ascii_case(text)
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
                let token = ArmOperand::ImmediateRange(u32::MAX, u32::MAX);
                let (arm, is_token) = match arm_of(operand) {
                    Some(arm) => (arm, false),
                    None => (&token, true),
                };
                match arm_predicate(self.target, &self.arm_class_predicate(class), arm) {
                    Diag::Match if is_token => ArmStatus::InvalidOperand,
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
        let short = self.arm_match_variant(operands, 1);
        if short.status == ArmStatus::Success {
            return short;
        }
        let long = self.arm_match_variant(operands, 0);
        if long.status == ArmStatus::InvalidOperand
            && long.error_index == Some(1)
            && operands.len() > 1
            && matches!(operands[1].kind, ParsedKind::Token(_))
            && self.arm_suffixes.contains(&operands[1].start)
        {
            return short;
        }
        long
    }

    fn arm_mnemonic_suggestion(&self, base: &str) -> String {
        const MAXIMUM_DISTANCE: usize = 2;
        let matcher = self.matcher;
        let target = self.target;
        let mut candidates: Vec<&str> = Vec::new();
        let mut previous = "";
        for entry in matcher.matchables.iter().filter(|entry| entry.variant == 0) {
            let mut predicates = target.instructions[entry.result].predicates.clone();
            if let Some(alias) = entry.alias {
                predicates.extend(target.aliases[alias].predicates.clone());
            }
            if !predicates_hold(target, &predicates, &self.arm_features) {
                continue;
            }
            let text = entry.mnemonic.as_str();
            if text == previous {
                continue;
            }
            previous = text;
            if edit_distance_without_replacements(base, text) <= MAXIMUM_DISTANCE {
                candidates.push(text);
            }
        }
        match candidates.split_last() {
            None => String::new(),
            Some((last, rest)) => {
                let mut text = ", did you mean: ".to_string();
                for candidate in rest {
                    text.push_str(candidate);
                    text.push_str(", ");
                }
                text.push_str(last);
                text.push('?');
                text
            }
        }
    }

    fn arm_match_variant(&self, operands: &[ParsedOperand], variant: usize) -> ArmOutcome {
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
        let Some((start, end)) = matcher.ranges.get(&(variant, mnemonic.clone())).copied() else {
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

    fn arm_register_key(&self, register: RegisterId) -> String {
        let name = self.target.registers[register as usize]
            .assembly_name
            .to_ascii_lowercase();
        match name.as_str() {
            "sp" | "wsp" => "sp".to_string(),
            "xzr" | "wzr" => "zr".to_string(),
            _ => name
                .strip_prefix(['x', 'w'])
                .map(|rest| rest.to_string())
                .unwrap_or(name),
        }
    }

    fn arm_validate(
        &mut self,
        operands: &[ParsedOperand],
        instruction: &MachineInstruction,
    ) -> bool {
        let name = self.target.instructions[instruction.opcode].record.clone();
        let key = |this: &Self, index: usize| this.arm_register_key(instruction.register_at(index));
        let place = |this: &Self, index: usize| {
            operands
                .get(index + 1)
                .map(|operand| this.statement_location(operand.start))
                .unwrap_or(this.statement_start)
        };
        let writeback = |name: &str, stems: &[&str]| -> bool {
            stems.iter().any(|stem| {
                name.strip_prefix(stem)
                    .is_some_and(|rest| rest == "pre" || rest == "post")
            })
        };
        let letters = |rest: &str, allowed: &[&str]| allowed.contains(&rest);
        let flags = [
            "", "WN", "RN", "N", "WT", "WTWN", "WTRN", "WTN", "RT", "RTWN", "RTRN", "RTN", "T",
            "TWN", "TRN", "TN",
        ];
        let location_zero = place(self, 0);
        let location_one = place(self, 1);
        if matches!(
            name.as_str(),
            "ADDSWri"
                | "ADDSXri"
                | "ADDWri"
                | "ADDXri"
                | "SUBSWri"
                | "SUBSXri"
                | "SUBWri"
                | "SUBXri"
        ) {
            if let Some(MachineOperand::Expression(expression)) = instruction.operands.get(2) {
                if let Some((specifier, _)) = classify_symbol(expression) {
                    let allowed = specifier.as_deref().is_some_and(|held| {
                        matches!(
                            held,
                            "lo12"
                                | "got_auth_lo12"
                                | "dtprel_hi12"
                                | "dtprel_lo12"
                                | "dtprel_lo12_nc"
                                | "tprel_hi12"
                                | "tprel_lo12"
                                | "tprel_lo12_nc"
                                | "tlsdesc_lo12"
                                | "tlsdesc_auth_lo12"
                                | "secrel_lo12"
                                | "secrel_hi12"
                        )
                    }) && matches!(name.as_str(), "ADDXri" | "ADDWri");
                    if !allowed {
                        let last = place(self, operands.len().saturating_sub(2));
                        return self.error(last, "invalid immediate expression");
                    }
                }
            }
        }
        if writeback(&name, &["LDPSW", "LDPW", "LDPX"]) {
            let base = key(self, 3);
            if base == key(self, 1) {
                return self.error(
                    location_zero,
                    "unpredictable LDP instruction, writeback base is also a destination",
                );
            }
            if base == key(self, 2) {
                return self.error(
                    location_one,
                    "unpredictable LDP instruction, writeback base is also a destination",
                );
            }
        }
        if matches!(
            name.as_str(),
            "LDPDi" | "LDPQi" | "LDPSi" | "LDPSWi" | "LDPWi" | "LDPXi"
        ) && instruction.register_at(0) == instruction.register_at(1)
        {
            return self.error(location_one, "unpredictable LDP instruction, Rt2==Rt");
        }
        if (writeback(&name, &["LDPD", "LDPQ", "LDPS"]) || name == "LDPSWpost")
            && instruction.register_at(1) == instruction.register_at(2)
        {
            return self.error(location_one, "unpredictable LDP instruction, Rt2==Rt");
        }
        if writeback(&name, &["STPD", "STPQ", "STPS", "STPW", "STPX"]) {
            let base = key(self, 3);
            if base == key(self, 1) {
                return self.error(
                    location_zero,
                    "unpredictable STP instruction, writeback base is also a source",
                );
            }
            if base == key(self, 2) {
                return self.error(
                    location_one,
                    "unpredictable STP instruction, writeback base is also a source",
                );
            }
        }
        if writeback(
            &name,
            &[
                "LDRBB", "LDRB", "LDRHH", "LDRH", "LDRSBW", "LDRSBX", "LDRSHW", "LDRSHX", "LDRSW",
                "LDRW", "LDRX",
            ],
        ) && key(self, 2) == key(self, 1)
        {
            return self.error(
                location_zero,
                "unpredictable LDR instruction, writeback base is also a source",
            );
        }
        if writeback(&name, &["STRBB", "STRB", "STRHH", "STRH", "STRW", "STRX"])
            && key(self, 2) == key(self, 1)
        {
            return self.error(
                location_zero,
                "unpredictable STR instruction, writeback base is also a source",
            );
        }
        if matches!(
            name.as_str(),
            "STXRB" | "STXRH" | "STXRW" | "STXRX" | "STLXRB" | "STLXRH" | "STLXRW" | "STLXRX"
        ) {
            let status = key(self, 0);
            let base = key(self, 2);
            if key(self, 1) == status || (base == status && base != "sp") {
                return self.error(
                    location_zero,
                    "unpredictable STXR instruction, status is also a source",
                );
            }
        }
        if matches!(name.as_str(), "STXPW" | "STXPX" | "STLXPW" | "STLXPX") {
            let status = key(self, 0);
            let base = key(self, 3);
            if key(self, 1) == status || key(self, 2) == status || (base == status && base != "sp")
            {
                return self.error(
                    location_zero,
                    "unpredictable STXP instruction, status is also a source",
                );
            }
        }
        if matches!(name.as_str(), "LDRABwriteback" | "LDRAAwriteback")
            && instruction.register_at(0) == instruction.register_at(1)
        {
            return self.error(
                location_zero,
                "unpredictable LDRA instruction, writeback base is also a destination",
            );
        }
        let copy_rest = name
            .strip_prefix("CPYF")
            .or_else(|| name.strip_prefix("CPY"))
            .and_then(|rest| rest.strip_prefix(['P', 'M', 'E']));
        if copy_rest.is_some_and(|rest| letters(rest, &flags)) {
            let destination = instruction.register_at(3);
            let source = instruction.register_at(4);
            let size = instruction.register_at(5);
            let text = if destination == source {
                "invalid CPY instruction, destination and source registers are the same"
            } else if destination == size {
                "invalid CPY instruction, destination and size registers are the same"
            } else if source == size {
                "invalid CPY instruction, source and size registers are the same"
            } else {
                ""
            };
            if !text.is_empty() {
                return self.error(location_zero, text);
            }
        }
        let set_rest = |name: &str| -> Option<String> {
            let rest = name
                .strip_prefix("SETGO")
                .map(|rest| ("O", rest))
                .or_else(|| name.strip_prefix("SETG").map(|rest| ("", rest)))
                .or_else(|| name.strip_prefix("SET").map(|rest| ("", rest)))
                .or_else(|| name.strip_prefix("MOPSSETG").map(|rest| ("", rest)))?;
            let tail = rest.1.strip_prefix(['P', 'M', 'E'])?;
            matches!(tail, "" | "T" | "N" | "TN").then(|| rest.0.to_string())
        };
        if name.starts_with("SETGO")
            || name.starts_with("SETG")
            || name.starts_with("SET")
            || name.starts_with("MOPSSETG")
        {
            if let Some(kind) = set_rest(&name) {
                let destination = instruction.register_at(2);
                let size = instruction.register_at(3);
                if destination == size {
                    return self.error(
                        location_zero,
                        "invalid SET instruction, destination and size registers are the same",
                    );
                }
                if kind.is_empty() {
                    let source = instruction.register_at(4);
                    if destination == source {
                        return self.error(
                            location_zero,
                            "invalid SET instruction, destination and source registers are the same",
                        );
                    }
                    if size == source {
                        return self.error(
                            location_zero,
                            "invalid SET instruction, source and size registers are the same",
                        );
                    }
                }
            }
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
                if self.arm_validate(operands, &instruction) {
                    return true;
                }
                let locations = std::mem::take(&mut self.expression_locations);
                self.streamer.note_expression_locations(&locations);
                self.emit_instruction(&instruction, location);
                false
            }
            ArmStatus::MissingFeature => {
                let mut text = "instruction requires:".to_string();
                let mut seen: Vec<String> = Vec::new();
                for feature in &outcome.missing {
                    if !seen.contains(feature) {
                        seen.push(feature.clone());
                    }
                }
                seen.sort();
                for feature in &seen {
                    let name = match self.target.records.get(feature) {
                        Some(record) if !table_text(record, "PredicateName").is_empty() => {
                            table_text(record, "PredicateName")
                        }
                        Some(record) => match record.field("AssemblerCondDag") {
                            Some(condition) => requirement_text(self.target, condition).0,
                            None => feature.clone(),
                        },
                        None => feature.clone(),
                    };
                    text.push(' ');
                    text.push_str(&name);
                }
                self.error(location, &text)
            }
            ArmStatus::MnemonicFail => {
                let base = operands
                    .first()
                    .map(|operand| operand.token_text().to_string())
                    .unwrap_or_default();
                let suggestion = self.arm_mnemonic_suggestion(&base);
                self.error(
                    location,
                    &format!("unrecognized instruction mnemonic{suggestion}"),
                )
            }
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
                    let end = self.lexer.token.location();
                    return self.error_range(
                        location,
                        "too few operands for instruction",
                        (location, end),
                    );
                }
                let at = self.statement_location(operands[index].start);
                let suffix = matches!(&operands[index].kind, ParsedKind::Token(_))
                    && self.arm_suffixes.contains(&operands[index].start);
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
        if self.is(TokenKind::Identifier) && self.lexer.token.text.eq_ignore_ascii_case(".req") {
            return self.arm_define_register_alias(identifier, identifier_token.location());
        }
        self.arm_suffixes.clear();
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

pub fn elf_relocation(
    kind: FixupKind,
    specifier: &str,
    pc_relative: bool,
) -> Result<u32, &'static str> {
    let specifier = specifier.to_ascii_lowercase();
    let spec = specifier.as_str();
    let by_specifier =
        |table: &[(&str, u32)], message: &'static str| -> Result<u32, &'static str> {
            table
                .iter()
                .find(|(held, _)| *held == spec)
                .map(|(_, number)| *number)
                .ok_or(message)
        };
    match kind {
        FixupKind::Data1 => Err("1-byte data relocations not supported"),
        FixupKind::Data2 => Ok(if pc_relative { 262 } else { 259 }),
        FixupKind::Data4 => Ok(if pc_relative { 261 } else { 258 }),
        FixupKind::Data8 => Ok(if pc_relative {
            260
        } else if spec == "dtprel" {
            1029
        } else {
            257
        }),
        FixupKind::Arm(fixup) => match fixup {
            ArmFixup::PcrelAdrImm21 => by_specifier(
                &[("", 274), ("abs", 274)],
                "invalid symbol kind for ADR relocation",
            ),
            ArmFixup::PcrelAdrpImm21 => by_specifier(
                &[
                    ("", 275),
                    ("pg_hi21", 275),
                    ("pg_hi21_nc", 276),
                    ("got", 311),
                    ("gottprel", 541),
                    ("tlsdesc", 562),
                ],
                "invalid symbol kind for ADRP relocation",
            ),
            ArmFixup::PcrelBranch26 => Ok(282),
            ArmFixup::PcrelCall26 => Ok(283),
            ArmFixup::PcrelBranch14 => Ok(279),
            ArmFixup::PcrelBranch19 => Ok(280),
            ArmFixup::PcrelBranch16 => Err("relocation of PAC/AUT instructions is not supported"),
            ArmFixup::PcrelBranch9 => {
                Err("relocation of compare-and-branch instructions not supported")
            }
            ArmFixup::LdrPcrelImm19 => by_specifier(
                &[("", 273), ("gottprel", 543), ("got", 309)],
                "invalid symbol kind for LDR relocation",
            ),
            ArmFixup::AddImm12 => by_specifier(
                &[
                    ("dtprel_hi12", 528),
                    ("tprel_hi12", 549),
                    ("dtprel_lo12_nc", 530),
                    ("dtprel_lo12", 529),
                    ("tprel_lo12_nc", 551),
                    ("tprel_lo12", 550),
                    ("tlsdesc_lo12", 564),
                    ("lo12", 277),
                ],
                "invalid fixup for add (uimm12) instruction",
            ),
            ArmFixup::LdStImm12Scale1 => by_specifier(
                &[
                    ("lo12", 278),
                    ("dtprel_lo12", 531),
                    ("dtprel_lo12_nc", 532),
                    ("tprel_lo12", 552),
                    ("tprel_lo12_nc", 553),
                ],
                "invalid fixup for 8-bit load/store instruction",
            ),
            ArmFixup::LdStImm12Scale2 => by_specifier(
                &[
                    ("lo12", 284),
                    ("dtprel_lo12", 533),
                    ("dtprel_lo12_nc", 534),
                    ("tprel_lo12", 554),
                    ("tprel_lo12_nc", 555),
                ],
                "invalid fixup for 16-bit load/store instruction",
            ),
            ArmFixup::LdStImm12Scale4 if spec == "got_lo12" => {
                Err("4 byte unchecked GOT load/store relocation is not supported in LP64")
            }
            ArmFixup::LdStImm12Scale4 if spec == "gottprel_lo12" => {
                Err("32-bit load/store relocation is not supported in LP64")
            }
            ArmFixup::LdStImm12Scale4 if spec == "tlsdesc_lo12" => {
                Err("4 byte TLSDESC load/store relocation is not supported in LP64")
            }
            ArmFixup::LdStImm12Scale4 => by_specifier(
                &[
                    ("lo12", 285),
                    ("dtprel_lo12", 535),
                    ("dtprel_lo12_nc", 536),
                    ("tprel_lo12", 556),
                    ("tprel_lo12_nc", 557),
                ],
                "invalid fixup for 32-bit load/store instruction fixup_aarch64_ldst_imm12_scale4",
            ),
            ArmFixup::LdStImm12Scale8 => by_specifier(
                &[
                    ("lo12", 286),
                    ("got_lo12", 312),
                    ("gotpage_lo15", 313),
                    ("dtprel_lo12", 537),
                    ("dtprel_lo12_nc", 538),
                    ("tprel_lo12", 558),
                    ("tprel_lo12_nc", 559),
                    ("gottprel_lo12", 542),
                    ("tlsdesc_lo12", 563),
                ],
                "invalid fixup for 64-bit load/store instruction",
            ),
            ArmFixup::LdStImm12Scale16 => by_specifier(
                &[
                    ("lo12", 299),
                    ("dtprel_lo12", 572),
                    ("dtprel_lo12_nc", 573),
                    ("tprel_lo12", 570),
                    ("tprel_lo12_nc", 571),
                ],
                "invalid fixup for 128-bit load/store instruction",
            ),
            ArmFixup::Movw => by_specifier(
                &[
                    ("abs_g3", 269),
                    ("abs_g2", 267),
                    ("abs_g2_nc", 268),
                    ("abs_g2_s", 272),
                    ("abs_g1", 265),
                    ("abs_g1_nc", 266),
                    ("abs_g1_s", 271),
                    ("abs_g0", 263),
                    ("abs_g0_nc", 264),
                    ("abs_g0_s", 270),
                    ("prel_g3", 293),
                    ("prel_g2", 291),
                    ("prel_g2_nc", 292),
                    ("prel_g1", 289),
                    ("prel_g1_nc", 290),
                    ("prel_g0", 287),
                    ("prel_g0_nc", 288),
                    ("dtprel_g2", 523),
                    ("dtprel_g1", 524),
                    ("dtprel_g1_nc", 525),
                    ("dtprel_g0", 526),
                    ("dtprel_g0_nc", 527),
                    ("tprel_g2", 544),
                    ("tprel_g1", 545),
                    ("tprel_g1_nc", 546),
                    ("tprel_g0", 547),
                    ("tprel_g0_nc", 548),
                    ("gottprel_g1", 539),
                    ("gottprel_g0_nc", 540),
                ],
                "invalid fixup for movz/movk instruction",
            ),
        },
        _ => Err("Unsupported fixup kind"),
    }
}

fn edit_distance_without_replacements(from: &str, to: &str) -> usize {
    let from: Vec<char> = from.chars().collect();
    let to: Vec<char> = to.chars().collect();
    let mut row: Vec<usize> = (0..=to.len()).collect();
    for (y, left) in from.iter().enumerate() {
        let mut previous = row[0];
        row[0] = y + 1;
        for x in 1..=to.len() {
            let old = row[x];
            row[x] = if *left == to[x - 1] {
                previous
            } else {
                row[x - 1].min(row[x]) + 1
            };
            previous = old;
        }
    }
    row[to.len()]
}

pub const RELOCATION_NAMES: &[(&str, u32)] = &[
    ("R_AARCH64_NONE", 0),
    ("R_AARCH64_P32_ABS32", 1),
    ("R_AARCH64_P32_COPY", 180),
    ("R_AARCH64_P32_GLOB_DAT", 181),
    ("R_AARCH64_P32_JUMP_SLOT", 182),
    ("R_AARCH64_P32_RELATIVE", 183),
    ("R_AARCH64_P32_TLS_DTPMOD", 184),
    ("R_AARCH64_P32_TLS_DTPREL", 185),
    ("R_AARCH64_P32_TLS_TPREL", 186),
    ("R_AARCH64_P32_TLSDESC", 187),
    ("R_AARCH64_P32_IRELATIVE", 188),
    ("R_AARCH64_ABS64", 257),
    ("R_AARCH64_ABS32", 258),
    ("R_AARCH64_ABS16", 259),
    ("R_AARCH64_PREL64", 260),
    ("R_AARCH64_PREL32", 261),
    ("R_AARCH64_PREL16", 262),
    ("R_AARCH64_MOVW_UABS_G0", 263),
    ("R_AARCH64_MOVW_UABS_G0_NC", 264),
    ("R_AARCH64_MOVW_UABS_G1", 265),
    ("R_AARCH64_MOVW_UABS_G1_NC", 266),
    ("R_AARCH64_MOVW_UABS_G2", 267),
    ("R_AARCH64_MOVW_UABS_G2_NC", 268),
    ("R_AARCH64_MOVW_UABS_G3", 269),
    ("R_AARCH64_MOVW_SABS_G0", 270),
    ("R_AARCH64_MOVW_SABS_G1", 271),
    ("R_AARCH64_MOVW_SABS_G2", 272),
    ("R_AARCH64_LD_PREL_LO19", 273),
    ("R_AARCH64_ADR_PREL_LO21", 274),
    ("R_AARCH64_ADR_PREL_PG_HI21", 275),
    ("R_AARCH64_ADR_PREL_PG_HI21_NC", 276),
    ("R_AARCH64_ADD_ABS_LO12_NC", 277),
    ("R_AARCH64_LDST8_ABS_LO12_NC", 278),
    ("R_AARCH64_TSTBR14", 279),
    ("R_AARCH64_CONDBR19", 280),
    ("R_AARCH64_JUMP26", 282),
    ("R_AARCH64_CALL26", 283),
    ("R_AARCH64_LDST16_ABS_LO12_NC", 284),
    ("R_AARCH64_LDST32_ABS_LO12_NC", 285),
    ("R_AARCH64_LDST64_ABS_LO12_NC", 286),
    ("R_AARCH64_MOVW_PREL_G0", 287),
    ("R_AARCH64_MOVW_PREL_G0_NC", 288),
    ("R_AARCH64_MOVW_PREL_G1", 289),
    ("R_AARCH64_MOVW_PREL_G1_NC", 290),
    ("R_AARCH64_MOVW_PREL_G2", 291),
    ("R_AARCH64_MOVW_PREL_G2_NC", 292),
    ("R_AARCH64_MOVW_PREL_G3", 293),
    ("R_AARCH64_LDST128_ABS_LO12_NC", 299),
    ("R_AARCH64_MOVW_GOTOFF_G0", 300),
    ("R_AARCH64_MOVW_GOTOFF_G0_NC", 301),
    ("R_AARCH64_MOVW_GOTOFF_G1", 302),
    ("R_AARCH64_MOVW_GOTOFF_G1_NC", 303),
    ("R_AARCH64_MOVW_GOTOFF_G2", 304),
    ("R_AARCH64_MOVW_GOTOFF_G2_NC", 305),
    ("R_AARCH64_MOVW_GOTOFF_G3", 306),
    ("R_AARCH64_GOTREL64", 307),
    ("R_AARCH64_GOTREL32", 308),
    ("R_AARCH64_GOT_LD_PREL19", 309),
    ("R_AARCH64_LD64_GOTOFF_LO15", 310),
    ("R_AARCH64_ADR_GOT_PAGE", 311),
    ("R_AARCH64_LD64_GOT_LO12_NC", 312),
    ("R_AARCH64_LD64_GOTPAGE_LO15", 313),
    ("R_AARCH64_TLSGD_ADR_PREL21", 512),
    ("R_AARCH64_TLSGD_ADR_PAGE21", 513),
    ("R_AARCH64_TLSGD_ADD_LO12_NC", 514),
    ("R_AARCH64_TLSGD_MOVW_G1", 515),
    ("R_AARCH64_TLSGD_MOVW_G0_NC", 516),
    ("R_AARCH64_TLSLD_ADR_PREL21", 517),
    ("R_AARCH64_TLSLD_ADR_PAGE21", 518),
    ("R_AARCH64_TLSLD_ADD_LO12_NC", 519),
    ("R_AARCH64_TLSLD_MOVW_G1", 520),
    ("R_AARCH64_TLSLD_MOVW_G0_NC", 521),
    ("R_AARCH64_TLSLD_LD_PREL19", 522),
    ("R_AARCH64_TLSLD_MOVW_DTPREL_G2", 523),
    ("R_AARCH64_TLSLD_MOVW_DTPREL_G1", 524),
    ("R_AARCH64_TLSLD_MOVW_DTPREL_G1_NC", 525),
    ("R_AARCH64_TLSLD_MOVW_DTPREL_G0", 526),
    ("R_AARCH64_TLSLD_MOVW_DTPREL_G0_NC", 527),
    ("R_AARCH64_TLSLD_ADD_DTPREL_HI12", 528),
    ("R_AARCH64_TLSLD_ADD_DTPREL_LO12", 529),
    ("R_AARCH64_TLSLD_ADD_DTPREL_LO12_NC", 530),
    ("R_AARCH64_TLSLD_LDST8_DTPREL_LO12", 531),
    ("R_AARCH64_TLSLD_LDST8_DTPREL_LO12_NC", 532),
    ("R_AARCH64_TLSLD_LDST16_DTPREL_LO12", 533),
    ("R_AARCH64_TLSLD_LDST16_DTPREL_LO12_NC", 534),
    ("R_AARCH64_TLSLD_LDST32_DTPREL_LO12", 535),
    ("R_AARCH64_TLSLD_LDST32_DTPREL_LO12_NC", 536),
    ("R_AARCH64_TLSLD_LDST64_DTPREL_LO12", 537),
    ("R_AARCH64_TLSLD_LDST64_DTPREL_LO12_NC", 538),
    ("R_AARCH64_TLSIE_MOVW_GOTTPREL_G1", 539),
    ("R_AARCH64_TLSIE_MOVW_GOTTPREL_G0_NC", 540),
    ("R_AARCH64_TLSIE_ADR_GOTTPREL_PAGE21", 541),
    ("R_AARCH64_TLSIE_LD64_GOTTPREL_LO12_NC", 542),
    ("R_AARCH64_TLSIE_LD_GOTTPREL_PREL19", 543),
    ("R_AARCH64_TLSLE_MOVW_TPREL_G2", 544),
    ("R_AARCH64_TLSLE_MOVW_TPREL_G1", 545),
    ("R_AARCH64_TLSLE_MOVW_TPREL_G1_NC", 546),
    ("R_AARCH64_TLSLE_MOVW_TPREL_G0", 547),
    ("R_AARCH64_TLSLE_MOVW_TPREL_G0_NC", 548),
    ("R_AARCH64_TLSLE_ADD_TPREL_HI12", 549),
    ("R_AARCH64_TLSLE_ADD_TPREL_LO12", 550),
    ("R_AARCH64_TLSLE_ADD_TPREL_LO12_NC", 551),
    ("R_AARCH64_TLSLE_LDST8_TPREL_LO12", 552),
    ("R_AARCH64_TLSLE_LDST8_TPREL_LO12_NC", 553),
    ("R_AARCH64_TLSLE_LDST16_TPREL_LO12", 554),
    ("R_AARCH64_TLSLE_LDST16_TPREL_LO12_NC", 555),
    ("R_AARCH64_TLSLE_LDST32_TPREL_LO12", 556),
    ("R_AARCH64_TLSLE_LDST32_TPREL_LO12_NC", 557),
    ("R_AARCH64_TLSLE_LDST64_TPREL_LO12", 558),
    ("R_AARCH64_TLSLE_LDST64_TPREL_LO12_NC", 559),
    ("R_AARCH64_TLSDESC_LD_PREL19", 560),
    ("R_AARCH64_TLSDESC_ADR_PREL21", 561),
    ("R_AARCH64_TLSDESC_ADR_PAGE21", 562),
    ("R_AARCH64_TLSDESC_LD64_LO12", 563),
    ("R_AARCH64_TLSDESC_ADD_LO12", 564),
    ("R_AARCH64_TLSDESC_OFF_G1", 565),
    ("R_AARCH64_TLSDESC_OFF_G0_NC", 566),
    ("R_AARCH64_TLSDESC_LDR", 567),
    ("R_AARCH64_TLSDESC_ADD", 568),
    ("R_AARCH64_TLSDESC_CALL", 569),
    ("R_AARCH64_TLSLE_LDST128_TPREL_LO12", 570),
    ("R_AARCH64_TLSLE_LDST128_TPREL_LO12_NC", 571),
    ("R_AARCH64_TLSLD_LDST128_DTPREL_LO12", 572),
    ("R_AARCH64_TLSLD_LDST128_DTPREL_LO12_NC", 573),
    ("R_AARCH64_COPY", 1024),
    ("R_AARCH64_GLOB_DAT", 1025),
    ("R_AARCH64_JUMP_SLOT", 1026),
    ("R_AARCH64_RELATIVE", 1027),
    ("R_AARCH64_TLS_DTPMOD64", 1028),
    ("R_AARCH64_TLS_DTPREL64", 1029),
    ("R_AARCH64_TLS_TPREL64", 1030),
    ("R_AARCH64_TLSDESC", 1031),
    ("R_AARCH64_IRELATIVE", 1032),
    ("R_AARCH64_PLT32", 314),
    ("R_AARCH64_GOTPCREL32", 315),
    ("BFD_RELOC_NONE", 0),
    ("BFD_RELOC_16", 259),
    ("BFD_RELOC_32", 258),
    ("BFD_RELOC_64", 257),
];

pub fn relocation_number(name: &str) -> Option<u32> {
    RELOCATION_NAMES
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
}

pub fn fixup_bits(kind: FixupKind) -> (usize, usize) {
    match kind {
        FixupKind::Arm(fixup) => match fixup {
            ArmFixup::PcrelAdrImm21 | ArmFixup::PcrelAdrpImm21 => (0, 32),
            ArmFixup::AddImm12
            | ArmFixup::LdStImm12Scale1
            | ArmFixup::LdStImm12Scale2
            | ArmFixup::LdStImm12Scale4
            | ArmFixup::LdStImm12Scale8
            | ArmFixup::LdStImm12Scale16 => (10, 12),
            ArmFixup::LdrPcrelImm19 | ArmFixup::PcrelBranch19 => (5, 19),
            ArmFixup::Movw | ArmFixup::PcrelBranch16 => (5, 16),
            ArmFixup::PcrelBranch9 => (5, 9),
            ArmFixup::PcrelBranch14 => (5, 14),
            ArmFixup::PcrelBranch26 | ArmFixup::PcrelCall26 => (0, 26),
        },
        other => (0, other.size() * 8),
    }
}

pub fn fixup_target_offset(kind: FixupKind) -> u32 {
    match kind {
        FixupKind::Arm(fixup) => match fixup {
            ArmFixup::PcrelAdrImm21
            | ArmFixup::PcrelAdrpImm21
            | ArmFixup::PcrelBranch26
            | ArmFixup::PcrelCall26 => 0,
            ArmFixup::AddImm12
            | ArmFixup::LdStImm12Scale1
            | ArmFixup::LdStImm12Scale2
            | ArmFixup::LdStImm12Scale4
            | ArmFixup::LdStImm12Scale8
            | ArmFixup::LdStImm12Scale16 => 10,
            ArmFixup::LdrPcrelImm19
            | ArmFixup::Movw
            | ArmFixup::PcrelBranch9
            | ArmFixup::PcrelBranch14
            | ArmFixup::PcrelBranch16
            | ArmFixup::PcrelBranch19 => 5,
        },
        _ => 0,
    }
}

pub fn adjust_fixup_value(
    kind: FixupKind,
    specifier: &str,
    value: u64,
) -> Result<u64, &'static str> {
    let signed = value as i64;
    let fits = |bits: u32| -> bool {
        let limit = 1i64 << (bits - 1);
        signed >= -limit && signed < limit
    };
    let adr_bits = |value: u64| -> u64 {
        let low = value & 3;
        let high = (value & 0x1f_fffc) >> 2;
        (high << 5) | (low << 29)
    };
    let FixupKind::Arm(fixup) = kind else {
        return Ok(value);
    };
    match fixup {
        ArmFixup::PcrelAdrImm21 => {
            if !fits(21) {
                return Err("fixup value out of range");
            }
            Ok(adr_bits(value & 0x1f_ffff))
        }
        ArmFixup::PcrelAdrpImm21 => Ok(adr_bits((value & 0x1_ffff_f000) >> 12)),
        ArmFixup::LdrPcrelImm19 | ArmFixup::PcrelBranch19 => {
            if !fits(21) {
                return Err("fixup value out of range");
            }
            if value & 3 != 0 {
                return Err("fixup not sufficiently aligned");
            }
            Ok((value >> 2) & 0x7_ffff)
        }
        ArmFixup::AddImm12 | ArmFixup::LdStImm12Scale1 => {
            if value >> 12 != 0 {
                return Err("fixup value out of range");
            }
            Ok(value)
        }
        ArmFixup::LdStImm12Scale2
        | ArmFixup::LdStImm12Scale4
        | ArmFixup::LdStImm12Scale8
        | ArmFixup::LdStImm12Scale16 => {
            let shift = match fixup {
                ArmFixup::LdStImm12Scale2 => 1,
                ArmFixup::LdStImm12Scale4 => 2,
                ArmFixup::LdStImm12Scale8 => 3,
                _ => 4,
            };
            if value >> (12 + shift) != 0 {
                return Err("fixup value out of range");
            }
            if value & ((1 << shift) - 1) != 0 {
                return Err(match shift {
                    1 => "fixup must be 2-byte aligned",
                    2 => "fixup must be 4-byte aligned",
                    3 => "fixup must be 8-byte aligned",
                    _ => "fixup must be 16-byte aligned",
                });
            }
            Ok(value >> shift)
        }
        ArmFixup::Movw => {
            let spec = specifier.to_ascii_lowercase();
            let group = |name: &str| -> u32 {
                if name.contains("_g3") {
                    3
                } else if name.contains("_g2") {
                    2
                } else if name.contains("_g1") {
                    1
                } else {
                    0
                }
            };
            let absolute = spec.starts_with("abs_g");
            let signed_absolute = absolute && spec.ends_with("_s");
            if !absolute {
                if spec.is_empty() {
                    if !(-0xffff..=0xffff).contains(&signed) {
                        return Err("fixup value out of range [-0xFFFF, 0xFFFF]");
                    }
                    return Ok(if signed < 0 { !signed as u64 } else { value });
                }
                return Err("relocation for a thread-local variable points to an absolute symbol");
            }
            let shift = group(&spec) * 16;
            if signed_absolute {
                let shifted = signed >> shift;
                if !(-0xffff..=0xffff).contains(&shifted) {
                    return Err("fixup value out of range");
                }
                return Ok(if shifted < 0 {
                    !shifted as u64
                } else {
                    shifted as u64
                });
            }
            let shifted = value >> shift;
            if spec.ends_with("_nc") {
                Ok(shifted & 0xffff)
            } else if shifted > 0xffff {
                Err("fixup value out of range")
            } else {
                Ok(shifted)
            }
        }
        ArmFixup::PcrelBranch9 => {
            if !fits(11) {
                return Err("fixup value out of range");
            }
            if value & 3 != 0 {
                return Err("fixup not sufficiently aligned");
            }
            Ok((value >> 2) & 0x1ff)
        }
        ArmFixup::PcrelBranch14 => {
            if !fits(16) {
                return Err("fixup value out of range");
            }
            if value & 3 != 0 {
                return Err("fixup not sufficiently aligned");
            }
            Ok((value >> 2) & 0x3fff)
        }
        ArmFixup::PcrelBranch16 => {
            let negated = signed.wrapping_neg();
            if !(0..=((1 << 18) - 1)).contains(&negated) {
                return Err("fixup value out of range");
            }
            if negated & 3 != 0 {
                return Err("fixup not sufficiently aligned");
            }
            Ok(((negated as u64) >> 2) & 0xffff)
        }
        ArmFixup::PcrelBranch26 | ArmFixup::PcrelCall26 => {
            if !fits(28) {
                return Err("fixup value out of range");
            }
            if value & 3 != 0 {
                return Err("fixup not sufficiently aligned");
            }
            Ok((value >> 2) & 0x3ff_ffff)
        }
    }
}

pub fn decode_logical_immediate(value: u64, register_size: u32) -> u64 {
    let n = (value >> 12) & 1;
    let immr = (value >> 6) & 0x3f;
    let imms = value & 0x3f;
    let pattern_bits = ((n << 6) | (!imms & 0x3f)) as u32;
    let length = 31 - pattern_bits.leading_zeros();
    let mut size = 1u32 << length;
    let rotation = immr & (u64::from(size) - 1);
    let ones = imms & (u64::from(size) - 1);
    let mut pattern = if ones + 1 >= 64 {
        u64::MAX
    } else {
        (1u64 << (ones + 1)) - 1
    };
    for _ in 0..rotation {
        pattern = ((pattern & 1) << (size - 1)) | (pattern >> 1);
    }
    while size != register_size {
        pattern |= pattern << size;
        size *= 2;
    }
    pattern
}

fn any_movw_alias(value: u64, width: u32) -> bool {
    (0..=(width - 16))
        .step_by(16)
        .any(|shift| is_movz_alias(value, shift, width) || is_movn_alias(value, shift, width))
}

pub struct PrintedInstruction {
    pub text: String,
    pub comment: String,
}

fn arm_print_tables() -> &'static PrintTables {
    static CELL: OnceLock<PrintTables> = OnceLock::new();
    CELL.get_or_init(|| {
        let target = target().expect("the ARMv8 tables load");
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

fn c_hex(value: u64) -> String {
    if value == 0 {
        "0".to_string()
    } else {
        format!("{value:#x}")
    }
}

thread_local! {
    static PRINT_HEX: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn format_immediate(value: i64) -> String {
    if PRINT_HEX.with(std::cell::Cell::get) {
        if value < 0 {
            format!("-{}", c_hex(value.unsigned_abs()))
        } else {
            c_hex(value as u64)
        }
    } else {
        value.to_string()
    }
}

fn arm_expression_text(target: &Target, expression: &Expression) -> String {
    expression_text(target, expression, 0, Syntax::Att)
}

pub struct ArmPrinter<'t> {
    pub target: &'t Target,
    pub features: &'t BTreeSet<String>,
    pub comment: std::cell::RefCell<String>,
    pub hexadecimal: bool,
}

impl<'t> ArmPrinter<'t> {
    pub fn new(target: &'t Target, features: &'t BTreeSet<String>) -> ArmPrinter<'t> {
        ArmPrinter {
            target,
            features,
            comment: std::cell::RefCell::new(String::new()),
            hexadecimal: false,
        }
    }

    fn register_name(&self, register: RegisterId) -> String {
        self.target.registers[register as usize]
            .assembly_name
            .clone()
    }

    fn vector_register_name(&self, register: RegisterId) -> String {
        let held = &self.target.registers[register as usize];
        if held.record.starts_with('D') || held.record.starts_with('Q') {
            format!("v{}", held.encoding)
        } else {
            held.assembly_name.clone()
        }
    }

    fn register_text(&self, instruction: &MachineInstruction, index: usize) -> String {
        match instruction.operand(index) {
            MachineOperand::Register(register) => self.register_name(*register),
            _ => String::new(),
        }
    }

    pub fn print(&self, instruction: &MachineInstruction) -> String {
        PRINT_HEX.with(|held| held.set(self.hexadecimal));
        if let Some(text) = self.special(instruction) {
            return text;
        }
        let tables = arm_print_tables();
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
                if !predicates_hold(self.target, &predicates, self.features) {
                    continue;
                }
            }
            let mut index = 0;
            let matched = pattern.conditions.iter().all(|condition| {
                let operand = instruction.operand(index);
                index += 1;
                match condition {
                    AliasCondition::Ignore => true,
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
                        MachineOperand::Register(held) => self.target.in_register_class(class, *held),
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

    fn special(&self, instruction: &MachineInstruction) -> Option<String> {
        let record = self.target.instructions[instruction.opcode].record.as_str();
        if record == "TLSDESCCALL" || record == "TLSAUTHDESCCALL" {
            if let MachineOperand::Expression(expression) = instruction.operand(0) {
                return Some(format!(
                    "\t.{} {}",
                    record.to_ascii_lowercase(),
                    arm_expression_text(self.target, expression)
                ));
            }
        }
        if record == "SYSxt" {
            if let Some(text) = self.sys_alias(instruction) {
                return Some(text);
            }
        }
        if record == "PRFMroX" || record == "PRFMroW" {
            if let Some(text) = self.range_prefetch_alias(instruction, record == "PRFMroX") {
                return Some(text);
            }
        }
        if record == "SYSLxt" {
            if let Some(text) = self.sysl_alias(instruction) {
                return Some(text);
            }
        }
        if record == "SYSPxt" || record == "SYSPxt_XZR" {
            if let Some(text) = self.sysp_alias(instruction) {
                return Some(text);
            }
        }
        let immediate = |index: usize| match instruction.operand(index) {
            MachineOperand::Immediate(value) => Some(*value),
            _ => None,
        };
        let is_expression =
            |index: usize| matches!(instruction.operand(index), MachineOperand::Expression(_));
        match record {
            "SBFMXri" | "SBFMWri" | "UBFMXri" | "UBFMWri" => {
                let signed = record.starts_with("SBFM");
                let wide = record.contains('X');
                let (Some(immr), Some(imms)) = (immediate(2), immediate(3)) else {
                    return None;
                };
                let destination = self.register_text(instruction, 0);
                if immr == 0 {
                    let mnemonic = match imms {
                        7 if signed => Some("sxtb"),
                        7 if !wide => Some("uxtb"),
                        15 if signed => Some("sxth"),
                        15 if !wide => Some("uxth"),
                        31 if wide && signed => Some("sxtw"),
                        _ => None,
                    };
                    if let Some(mnemonic) = mnemonic {
                        let source = self.target.arm_w_from_x(instruction.register_at(1));
                        let source = if wide || mnemonic == "sxtw" {
                            self.register_name(source)
                        } else {
                            self.register_text(instruction, 1)
                        };
                        return Some(format!("\t{mnemonic}\t{destination}, {source}"));
                    }
                }
                let source = self.register_text(instruction, 1);
                let shift_form = if record == "UBFMWri" && imms != 0x1f && imms + 1 == immr {
                    Some(("lsl", 31 - imms))
                } else if record == "UBFMXri" && imms != 0x3f && imms + 1 == immr {
                    Some(("lsl", 63 - imms))
                } else if (record == "UBFMWri" && imms == 0x1f)
                    || (record == "UBFMXri" && imms == 0x3f)
                {
                    Some(("lsr", immr))
                } else if (record == "SBFMWri" && imms == 0x1f)
                    || (record == "SBFMXri" && imms == 0x3f)
                {
                    Some(("asr", immr))
                } else {
                    None
                };
                if let Some((mnemonic, shift)) = shift_form {
                    return Some(format!("\t{mnemonic}\t{destination}, {source}, #{shift}"));
                }
                if immr > imms {
                    let width = if wide { 64 } else { 32 };
                    return Some(format!(
                        "\t{}\t{destination}, {source}, #{}, #{}",
                        if signed { "sbfiz" } else { "ubfiz" },
                        width - immr,
                        imms + 1
                    ));
                }
                Some(format!(
                    "\t{}\t{destination}, {source}, #{immr}, #{}",
                    if signed { "sbfx" } else { "ubfx" },
                    imms - immr + 1
                ))
            }
            "BFMXri" | "BFMWri" => {
                let (Some(immr), Some(imms)) = (immediate(3), immediate(4)) else {
                    return None;
                };
                let width = if record == "BFMXri" { 64 } else { 32 };
                let zero = matches!(
                    self.target.registers[instruction.register_at(2) as usize]
                        .record
                        .as_str(),
                    "WZR" | "XZR"
                );
                let destination = self.register_text(instruction, 0);
                let source = self.register_text(instruction, 2);
                if zero && (immr == 0 || imms < immr) && self.features.contains("HasV8_2aOps") {
                    return Some(format!(
                        "\tbfc\t{destination}, #{}, #{}",
                        (width - immr) % width,
                        imms + 1
                    ));
                }
                if imms < immr {
                    return Some(format!(
                        "\tbfi\t{destination}, {source}, #{}, #{}",
                        (width - immr) % width,
                        imms + 1
                    ));
                }
                Some(format!(
                    "\tbfxil\t{destination}, {source}, #{immr}, #{}",
                    imms - immr + 1
                ))
            }
            "MOVZXi" | "MOVZWi" | "MOVNXi" | "MOVNWi" if is_expression(1) => {
                let mnemonic = if record.starts_with("MOVZ") {
                    "movz"
                } else {
                    "movn"
                };
                let MachineOperand::Expression(expression) = instruction.operand(1) else {
                    return None;
                };
                Some(format!(
                    "\t{mnemonic}\t{}, #{}",
                    self.register_text(instruction, 0),
                    arm_expression_text(self.target, expression)
                ))
            }
            "MOVKXi" | "MOVKWi" if is_expression(2) => {
                let MachineOperand::Expression(expression) = instruction.operand(2) else {
                    return None;
                };
                Some(format!(
                    "\tmovk\t{}, #{}",
                    self.register_text(instruction, 0),
                    arm_expression_text(self.target, expression)
                ))
            }
            "MOVZXi" | "MOVZWi" => {
                let (Some(value), Some(shift)) = (immediate(1), immediate(2)) else {
                    return None;
                };
                let width = if record == "MOVZXi" { 64 } else { 32 };
                let full = (value as u64).wrapping_shl(shift as u32);
                if is_movz_alias(full, shift as u32, width) {
                    return Some(self.mov_immediate(instruction, full, width));
                }
                None
            }
            "MOVNXi" | "MOVNWi" => {
                let (Some(value), Some(shift)) = (immediate(1), immediate(2)) else {
                    return None;
                };
                let width = if record == "MOVNXi" { 64 } else { 32 };
                let mut full = !((value as u64).wrapping_shl(shift as u32));
                if width == 32 {
                    full &= 0xffff_ffff;
                }
                if is_movn_alias(full, shift as u32, width) {
                    return Some(self.mov_immediate(instruction, full, width));
                }
                None
            }
            "ORRXri" | "ORRWri" => {
                let zero = matches!(
                    self.target.registers[instruction.register_at(1) as usize]
                        .record
                        .as_str(),
                    "WZR" | "XZR"
                );
                let value = immediate(2)?;
                if !zero {
                    return None;
                }
                let width = if record == "ORRXri" { 64 } else { 32 };
                let decoded = decode_logical_immediate(value as u64, width);
                if !any_movw_alias(decoded, width) {
                    return Some(self.mov_immediate(instruction, decoded, width));
                }
                None
            }
            _ => None,
        }
    }

    fn mov_immediate(&self, instruction: &MachineInstruction, value: u64, width: u32) -> String {
        let extended = if width == 64 {
            value as i64
        } else {
            (value as u32) as i32 as i64
        };
        let mask = if width == 64 {
            u64::MAX
        } else {
            (1u64 << width) - 1
        };
        if self.hexadecimal {
            self.comment
                .borrow_mut()
                .push_str(&format!("={extended}\n"));
        } else {
            self.comment
                .borrow_mut()
                .push_str(&format!("={:#x}\n", extended as u64 & mask));
        }
        format!(
            "\tmov\t{}, #{}",
            self.register_text(instruction, 0),
            format_immediate(extended)
        )
    }

    fn call_method(
        &self,
        method: &str,
        instruction: &MachineInstruction,
        index: usize,
        out: &mut String,
    ) {
        let (name, arguments) = split_method(method);
        let operand = instruction.operand(index);
        let number = |at: usize| argument_number(&arguments, at);
        let scaled_immediate = |scale: i64| match operand {
            MachineOperand::Immediate(value) => format!("#{}", format_immediate(value * scale)),
            MachineOperand::Expression(expression) => arm_expression_text(self.target, expression),
            _ => String::new(),
        };
        match name.as_str() {
            "" | "printOperand" => match operand {
                MachineOperand::Register(register) => out.push_str(&self.register_name(*register)),
                MachineOperand::Immediate(value) => {
                    out.push_str(&format!("#{}", format_immediate(*value)))
                }
                MachineOperand::Expression(expression) => {
                    out.push_str(&arm_expression_text(self.target, expression))
                }
            },
            "printImm" => out.push_str(&format!("#{}", format_immediate(operand.immediate()))),
            "printImmHex" => out.push_str(&format!("#{}", c_hex(operand.immediate() as u64))),
            "printSImm" => {
                let value = operand.immediate();
                let value = match number(0) {
                    8 => i64::from(value as i8),
                    16 => i64::from(value as i16),
                    _ => value,
                };
                out.push_str(&format!("#{}", format_immediate(value)))
            }
            "printPostIncOperand" => {
                if let MachineOperand::Register(register) = operand {
                    if self.target.registers[*register as usize].record == "XZR" {
                        out.push_str(&format!("#{}", number(0)));
                    } else {
                        out.push_str(&self.register_name(*register));
                    }
                }
            }
            "printVRegOperand" => {
                if let MachineOperand::Register(register) = operand {
                    out.push_str(&self.vector_register_name(*register));
                }
            }
            "printSysCROperand" => out.push_str(&format!("c{}", operand.immediate())),
            "printAddSubImm" => match operand {
                MachineOperand::Immediate(value) => {
                    let value = value & 0xfff;
                    let shift = instruction.immediate_at(index + 1) & 0x3f;
                    out.push_str(&format!("#{}", format_immediate(value)));
                    if shift != 0 {
                        self.shifter(instruction.immediate_at(index + 1), out);
                        self.comment
                            .borrow_mut()
                            .push_str(&format!("={}\n", format_immediate(value << shift)));
                    }
                }
                MachineOperand::Expression(expression) => {
                    out.push_str(&arm_expression_text(self.target, expression));
                    self.shifter(instruction.immediate_at(index + 1), out);
                }
                _ => {}
            },
            "printLogicalImm" => {
                let width = match arguments.first().map(String::as_str).unwrap_or("") {
                    text if text.contains("64") => 64,
                    text if text.contains("16") => 16,
                    text if text.contains('8') && !text.contains("32") => 8,
                    _ => 32,
                };
                out.push_str(&format!(
                    "#{:#x}",
                    decode_logical_immediate(operand.immediate() as u64, width)
                ));
            }
            "printShifter" => self.shifter(operand.immediate(), out),
            "printShiftedRegister" => {
                out.push_str(&self.register_text(instruction, index));
                self.shifter(instruction.immediate_at(index + 1), out);
            }
            "printExtendedRegister" => {
                out.push_str(&self.register_text(instruction, index));
                self.arith_extend(instruction, index + 1, out);
            }
            "printArithExtend" => self.arith_extend(instruction, index, out),
            "printMemExtend" => {
                let kind = arguments
                    .first()
                    .map(|text| text.trim_matches('\''))
                    .unwrap_or("x");
                let sign = operand.immediate() != 0;
                let shift = instruction.immediate_at(index + 1) != 0;
                self.mem_extend(sign, shift, number(1), kind, out);
            }
            "printRegWithShiftExtend" => {
                let sign = arguments.first().is_some_and(|text| text == "true");
                let width = number(1);
                let kind = arguments
                    .get(2)
                    .map(|text| text.trim_matches('\''))
                    .unwrap_or("x");
                let suffix = arguments
                    .get(3)
                    .map(|text| text.trim_matches('\''))
                    .unwrap_or("0");
                self.call_method("printOperand", instruction, index, out);
                if suffix == "s" || suffix == "d" {
                    out.push('.');
                    out.push_str(suffix);
                }
                let shift = width != 8;
                if sign || shift || kind == "w" {
                    out.push_str(", ");
                    self.mem_extend(sign, shift, width, kind, out);
                }
            }
            "printCondCode" => out.push_str(condition_name(operand.immediate() as u32)),
            "printInverseCondCode" => {
                out.push_str(condition_name((operand.immediate() as u32) ^ 1))
            }
            "printAMNoIndex" => {
                out.push('[');
                out.push_str(&self.register_text(instruction, index));
                out.push(']');
            }
            "printImmScale" => out.push_str(&scaled_immediate(number(0))),
            "printImmRangeScale" => {
                let first = number(0) * operand.immediate();
                out.push_str(&format!(
                    "{}:{}",
                    format_immediate(first),
                    format_immediate(first + number(1))
                ));
            }
            "printUImm12Offset" => out.push_str(&scaled_immediate(number(0).max(1))),
            "printAMIndexedWB" => {
                out.push('[');
                out.push_str(&self.register_text(instruction, index));
                out.push_str(", ");
                out.push_str(&match instruction.operand(index + 1) {
                    MachineOperand::Immediate(value) => {
                        format!("#{}", format_immediate(value * number(0).max(1)))
                    }
                    MachineOperand::Expression(expression) => {
                        arm_expression_text(self.target, expression)
                    }
                    _ => String::new(),
                });
                out.push(']');
            }
            "printPrefetchOp" | "printRPRFMOperand" => {
                let value = operand.immediate() as u32;
                let tables = system_tables_for();
                let names = if name == "printPrefetchOp" {
                    &tables.prefetch_names
                } else {
                    &tables.range_prefetch_names
                };
                match names.get(&value) {
                    Some(held) => out.push_str(held),
                    None => out.push_str(&format!("#{}", format_immediate(i64::from(value)))),
                }
            }
            "printTIndexHintOp" => {
                let value = operand.immediate() as u32;
                let tables = system_tables_for();
                match tables.hints.values().find(|entry| entry.encoding == value) {
                    Some(entry) => out.push_str(&entry.name),
                    None => out.push_str(&format!("#{}", format_immediate(i64::from(value)))),
                }
            }
            "printFPImmOperand" => {
                let value = float_from_immediate(operand.immediate() as u32) as f32;
                out.push_str(&format!("#{:.8}", f64::from(value)));
            }
            "printGPRSeqPairsClassOperand" => {
                if let MachineOperand::Register(register) = operand {
                    let members = &self.target.registers[*register as usize].sub_registers;
                    let names: Vec<String> = members
                        .iter()
                        .map(|member| self.register_name(*member))
                        .collect();
                    out.push_str(&names.join(", "));
                }
            }
            "printImplicitlyTypedVectorList" => self.vector_list(instruction, index, "", out),
            "printTypedVectorList" => {
                let lanes = number(0);
                let kind = arguments
                    .get(1)
                    .map(|text| text.trim_matches('\''))
                    .unwrap_or("0");
                if kind == "0" {
                    self.vector_list(instruction, index, "", out);
                } else if lanes != 0 {
                    self.vector_list(instruction, index, &format!(".{lanes}{kind}"), out);
                } else {
                    self.vector_list(instruction, index, &format!(".{kind}"), out);
                }
            }
            "printVectorIndex" => {
                let scale = if arguments.is_empty() { 1 } else { number(0) };
                out.push_str(&format!("[{}]", scale * operand.immediate()))
            }
            "printAlignedLabel" => match operand {
                MachineOperand::Immediate(value) => {
                    out.push_str(&format!("#{}", format_immediate(value * 4)))
                }
                MachineOperand::Expression(expression) => match expression.constant_value() {
                    Some(value) => out.push_str(&format!("{:#x}", value as u64)),
                    None => out.push_str(&arm_expression_text(self.target, expression)),
                },
                _ => {}
            },
            "printAdrAdrpLabel" => match operand {
                MachineOperand::Immediate(value) => {
                    let record = self.target.instructions[instruction.opcode].record.as_str();
                    let offset = if record == "ADRP" {
                        value * 4096
                    } else {
                        *value
                    };
                    out.push_str(&format!("#{offset}"));
                }
                MachineOperand::Expression(expression) => {
                    out.push_str(&arm_expression_text(self.target, expression))
                }
                _ => {}
            },
            "printBarrierOption" => {
                let value = operand.immediate() as u32;
                let record = self.target.instructions[instruction.opcode].record.as_str();
                let named = if record == "ISB" {
                    (value == 15).then(|| "sy".to_string())
                } else {
                    system_tables_for().barrier_names.get(&value).cloned()
                };
                match named {
                    Some(held) => out.push_str(&held),
                    None => out.push_str(&format!("#{value}")),
                }
            }
            "printBarriernXSOption" => {
                let value = operand.immediate() as u32;
                let found = system_tables_for()
                    .barriers_no_xs
                    .values()
                    .find(|(entry, _)| entry.encoding == value);
                match found {
                    Some((entry, _)) => out.push_str(&entry.name),
                    None => out.push_str(&format!("#{value}")),
                }
            }
            "printMRSSystemRegister" | "printMSRSystemRegister" => {
                let value = operand.immediate() as u32;
                let read = name == "printMRSSystemRegister";
                let found = system_tables_for()
                    .registers
                    .values()
                    .filter(|entry| {
                        entry.encoding == value
                            && (if read {
                                entry.readable
                            } else {
                                entry.writeable
                            })
                            && predicates_features(&entry.requires, self.features)
                    })
                    .min_by_key(|entry| entry.order);
                match found {
                    Some(entry) => out.push_str(&entry.name),
                    None => out.push_str(&generic_register_string(value)),
                }
            }
            "printSystemPStateField" => {
                let value = operand.immediate() as u32;
                let tables = system_tables_for();
                let fifteen = tables.pstate_fifteen.values().find(|entry| {
                    entry.encoding == value && predicates_features(&entry.requires, self.features)
                });
                let one = tables.pstate_one.values().find(|entry| {
                    entry.encoding == value && predicates_features(&entry.requires, self.features)
                });
                match fifteen.or(one) {
                    Some(entry) => out.push_str(&entry.name),
                    None => out.push_str(&format!("#{value}")),
                }
            }
            "printSIMDType10Operand" => {
                let raw = operand.immediate() as u32;
                let value = (0..8).fold(0u64, |value, bit| {
                    if raw & (1 << bit) != 0 {
                        value | (0xffu64 << (bit * 8))
                    } else {
                        value
                    }
                });
                out.push_str(&format!("#{value:#018x}"));
            }
            "printComplexRotationOp" => {
                out.push_str(&format!("#{}", operand.immediate() * number(0) + number(1)))
            }
            "printGPR64as32" => {
                if let MachineOperand::Register(register) = operand {
                    out.push_str(&self.register_name(self.target.arm_w_from_x(*register)));
                }
            }
            "printGPR64x8" => {
                if let MachineOperand::Register(register) = operand {
                    let members = &self.target.registers[*register as usize].sub_registers;
                    match members.first() {
                        Some(first) => out.push_str(&self.register_name(*first)),
                        None => out.push_str(&self.register_name(*register)),
                    }
                }
            }
            "printSyspXzrPair" => out.push_str("xzr, xzr"),
            _ => self.call_method("printOperand", instruction, index, out),
        }
    }

    fn shifter(&self, value: i64, out: &mut String) {
        let kind = (value >> 6) & 7;
        let amount = value & 0x3f;
        if kind == 0 && amount == 0 {
            return;
        }
        let names = ["lsl", "lsr", "asr", "ror", "msl"];
        out.push_str(&format!(
            ", {} #{}",
            names.get(kind as usize).copied().unwrap_or("lsl"),
            amount
        ));
    }

    fn arith_extend(&self, instruction: &MachineInstruction, index: usize, out: &mut String) {
        let value = instruction.immediate_at(index);
        let extension = ((value >> 3) & 7) as usize;
        let amount = value & 7;
        let names = [
            "uxtb", "uxth", "uxtw", "uxtx", "sxtb", "sxth", "sxtw", "sxtx",
        ];
        let name = names[extension];
        if name == "uxtw" || name == "uxtx" {
            let destination = self.target.registers[instruction.register_at(0) as usize]
                .record
                .as_str();
            let source = self.target.registers[instruction.register_at(1) as usize]
                .record
                .as_str();
            if ((destination == "SP" || source == "SP") && name == "uxtx")
                || ((destination == "WSP" || source == "WSP") && name == "uxtw")
            {
                if amount != 0 {
                    out.push_str(&format!(", lsl #{amount}"));
                }
                return;
            }
        }
        out.push_str(&format!(", {name}"));
        if amount != 0 {
            out.push_str(&format!(" #{amount}"));
        }
    }

    fn mem_extend(&self, sign: bool, shift: bool, width: i64, kind: &str, out: &mut String) {
        let lsl = !sign && kind == "x";
        if lsl {
            out.push_str("lsl");
        } else {
            out.push_str(&format!("{}xt{kind}", if sign { 's' } else { 'u' }));
        }
        if shift || lsl {
            out.push_str(&format!(" #{}", (width / 8).max(1).ilog2()));
        }
    }

    fn vector_list(
        &self,
        instruction: &MachineInstruction,
        index: usize,
        suffix: &str,
        out: &mut String,
    ) {
        let MachineOperand::Register(register) = instruction.operand(index) else {
            return;
        };
        let held = &self.target.registers[*register as usize];
        let members: Vec<RegisterId> = if held.sub_registers.len() > 1 && held.record.contains('_')
        {
            held.sub_registers.clone()
        } else {
            vec![*register]
        };
        let names: Vec<String> = members
            .iter()
            .map(|member| format!("{}{suffix}", self.vector_register_name(*member)))
            .collect();
        out.push_str(&format!("{{ {} }}", names.join(", ")));
    }
}

fn generic_register_string(value: u32) -> String {
    let op0 = (value >> 14) & 3;
    let op1 = (value >> 11) & 7;
    let crn = (value >> 7) & 15;
    let crm = (value >> 3) & 15;
    let op2 = value & 7;
    format!("S{op0}_{op1}_C{crn}_C{crm}_{op2}")
}

fn extension_features(_target: &Target, name: &str) -> Option<Vec<String>> {
    EXTENSION_DEFS
        .iter()
        .find(|(held, _)| *held == name)
        .map(|(_, features)| features.iter().map(|feature| feature.to_string()).collect())
}

fn set_feature(target: &Target, features: &mut BTreeSet<String>, name: &str) {
    let mut pending = vec![name.to_string()];
    while let Some(held) = pending.pop() {
        if features.insert(held.clone()) {
            if let Some(record) = target.records.get(&held) {
                pending.extend(table_record_list(record, "Implies"));
            }
        }
    }
}

fn clear_feature(target: &Target, features: &mut BTreeSet<String>, name: &str) {
    let mut removed = BTreeSet::new();
    removed.insert(name.to_string());
    loop {
        let before = removed.len();
        for held in features.clone() {
            if let Some(record) = target.records.get(&held) {
                if table_record_list(record, "Implies")
                    .iter()
                    .any(|implied| removed.contains(implied))
                {
                    removed.insert(held);
                }
            }
        }
        if removed.len() == before {
            break;
        }
    }
    for held in removed {
        features.remove(&held);
    }
}

fn architecture_feature(target: &Target, name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let lower = {
        let body = lower
            .strip_prefix("armv")
            .or_else(|| lower.strip_prefix('v'))
            .unwrap_or(&lower);
        let digits = body.trim_end_matches(|character: char| character.is_ascii_alphabetic());
        let tail = &body[digits.len()..];
        if body.contains('-')
            || digits.is_empty()
            || !(tail.is_empty() || tail == "a" || tail == "r")
            || (tail.is_empty() && digits.contains('.'))
        {
            lower.clone()
        } else {
            format!("armv{digits}-{}", if tail.is_empty() { "a" } else { tail })
        }
    };
    target
        .records
        .values()
        .filter(|record| record.is_a("Architecture64"))
        .find(|record| {
            let major = table_integer(record, "Major");
            let minor = table_integer(record, "Minor");
            let profile = table_text(record, "Profile");
            let spelled = if minor == 0 {
                format!("armv{major}-{profile}")
            } else {
                format!("armv{major}.{minor}-{profile}")
            };
            spelled == lower
        })
        .map(|record| record.name.clone())
}

impl<'s> Assembler<'s> {
    fn arm_apply_extensions(&mut self, text: &str, location: SourceLocation) -> bool {
        let target = self.target;
        let mut offset = 0usize;
        for piece in text.split('+') {
            let position = SourceLocation {
                buffer: location.buffer,
                position: location.position + offset,
            };
            offset += piece.len() + 1;
            if piece.is_empty() {
                continue;
            }
            let lowered = piece.to_ascii_lowercase();
            let (enable, name) = match lowered.strip_prefix("no") {
                Some(rest) => (false, rest.to_string()),
                None => (true, lowered.clone()),
            };
            let names: Vec<String> = if name == "crypto" {
                vec!["FeatureAES".to_string(), "FeatureSHA2".to_string()]
            } else {
                match extension_features(target, &name) {
                    Some(found) => found,
                    None => {
                        return self.error(
                            position,
                            &format!("unsupported architectural extension: {name}"),
                        );
                    }
                }
            };
            for held in names {
                if enable {
                    set_feature(target, &mut self.arm_features, &held);
                } else {
                    clear_feature(target, &mut self.arm_features, &held);
                }
            }
        }
        false
    }

    pub(super) fn arm_directive(
        &mut self,
        identifier: &str,
        location: SourceLocation,
    ) -> Option<bool> {
        let result = self.arm_directive_inner(identifier, location);
        if matches!(identifier, ".arch" | ".cpu" | ".arch_extension") {
            self.streamer.set_arm_features(&self.arm_features);
        }
        result
    }

    fn arm_directive_inner(&mut self, identifier: &str, location: SourceLocation) -> Option<bool> {
        match identifier {
            ".aeabi_subsection" => Some(self.arm_directive_subsection()),
            ".aeabi_attribute" => Some(self.arm_directive_attribute()),
            ".hword" => Some(self.parse_directive_value(2)),
            ".word" => Some(self.parse_directive_value(4)),
            ".dword" | ".xword" => Some(self.parse_directive_value(8)),
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
            ".variant_pcs" => {
                let Some(name) = self.parse_identifier() else {
                    return Some(self.token_error("expected symbol name"));
                };
                if self.parse_end_of_line() {
                    return Some(true);
                }
                let symbol = self.context.symbol(&name);
                self.streamer.emit_variant_pcs(&mut self.context, symbol);
                Some(false)
            }
            ".tlsdesccall" | ".tlsauthdesccall" => {
                let record = if identifier == ".tlsdesccall" {
                    "TLSDESCCALL"
                } else {
                    "TLSAUTHDESCCALL"
                };
                let Some(name) = self.parse_identifier() else {
                    return Some(self.token_error("expected symbol name"));
                };
                if self.parse_end_of_line() {
                    return Some(true);
                }
                let Some(opcode) = self.target.instruction_by_record.get(record) else {
                    return Some(true);
                };
                let mut instruction = MachineInstruction::new(*opcode);
                instruction
                    .operands
                    .push(MachineOperand::Expression(Expression::symbol(&name)));
                self.emit_instruction(&instruction, location);
                Some(false)
            }
            ".inst" => {
                if self.is(TokenKind::EndOfStatement) {
                    return Some(
                        self.error(location, "expected expression following '.inst' directive"),
                    );
                }
                Some(self.parse_many(|parser| {
                    let at = parser.location();
                    let Ok((value, _)) = parser.parse_expression() else {
                        return true;
                    };
                    let Some(constant) = parser.context.evaluate_absolute(&value, None) else {
                        return parser.error(at, "expected constant expression");
                    };
                    if !(0..=0xffff_ffff).contains(&constant) && constant < -(1i64 << 31) {
                        return parser.error(at, "out of range literal value");
                    }
                    parser.streamer.emit_inst(&mut parser.context, constant, at);
                    false
                }))
            }
            ".arch" => {
                let at = self.location();
                let text = self.parse_string_to_end_of_statement();
                let text = text.trim().to_string();
                let (architecture, extensions) = match text.split_once('+') {
                    Some((head, rest)) => (head.to_string(), rest.to_string()),
                    None => (text.clone(), String::new()),
                };
                let target = self.target;
                let Some(feature) = architecture_feature(target, &architecture) else {
                    return Some(self.error(at, "unknown arch name"));
                };
                let mut features = BTreeSet::new();
                for held in feature_set(target, "", &[]) {
                    features.insert(held);
                }
                set_feature(target, &mut features, &feature);
                if let Some(record) = target.records.get(&feature) {
                    for default in table_record_list(record, "DefaultExts") {
                        set_feature(target, &mut features, &default);
                    }
                }
                self.arm_features = features;
                let shifted = SourceLocation {
                    buffer: at.buffer,
                    position: at.position + architecture.len() + 1,
                };
                Some(self.arm_apply_extensions(&extensions, shifted))
            }
            ".cpu" => {
                let at = self.location();
                let text = self.parse_string_to_end_of_statement();
                let text = text.trim().to_string();
                let (cpu, extensions) = match text.split_once('+') {
                    Some((head, rest)) => (head.to_string(), rest.to_string()),
                    None => (text.clone(), String::new()),
                };
                let target = self.target;
                let known = target.records.values().any(|record| {
                    record.is_a("ProcessorModel") && table_text(record, "Name") == cpu
                });
                if !known {
                    self.error(at, "unknown CPU name");
                    return Some(false);
                }
                self.arm_features = feature_set(target, &cpu, &[]);
                let shifted = SourceLocation {
                    buffer: at.buffer,
                    position: at.position + cpu.len() + 1,
                };
                Some(self.arm_apply_extensions(&extensions, shifted))
            }
            ".arch_extension" => {
                let at = self.location();
                let text = self.parse_string_to_end_of_statement();
                let text = text.trim().to_string();
                if self.parse_end_of_line() {
                    return Some(true);
                }
                Some(self.arm_apply_extensions(&text, at))
            }
            ".unreq" => {
                if !self.is(TokenKind::Identifier) {
                    return Some(self.token_error("unexpected input in .unreq directive."));
                }
                let name = self.lexer.token.text.to_ascii_lowercase();
                self.arm_aliases.remove(&name);
                self.lex();
                Some(self.parse_end_of_line())
            }
            _ => None,
        }
    }

    fn arm_define_register_alias(&mut self, name: &str, name_location: SourceLocation) -> bool {
        self.lex();
        if !self.is(TokenKind::Identifier) {
            return self.token_error("unexpected input in .req directive");
        }
        let text = self.lexer.token.text.clone();
        let (kind, register) =
            if let Some(register) = self.arm_register_by_name(&text, RegKind::Scalar) {
                (RegKind::Scalar, register)
            } else if let Some(register) = self.arm_register_by_name(&text, RegKind::NeonVector) {
                (RegKind::NeonVector, register)
            } else {
                let base = text.split('.').next().unwrap_or("");
                let typed = text.contains('.')
                    && self
                        .arm_register_by_name(base, RegKind::NeonVector)
                        .is_some();
                return self.token_error(if typed {
                    "vector register without type specifier expected"
                } else {
                    "register name or alias expected"
                });
            };
        self.lex();
        if self.parse_end_of_line() {
            return true;
        }
        let lower = name.to_ascii_lowercase();
        match self.arm_aliases.get(&lower) {
            Some((held, existing)) if *held != kind || *existing != register => {
                self.warning(
                    name_location,
                    &format!("ignoring redefinition of register alias '{name}'"),
                );
            }
            _ => {
                self.arm_aliases.insert(lower, (kind, register));
            }
        }
        false
    }
}

const EXTENSION_DEFS: &[(&str, &[&str])] = &[
    ("crc", &["FeatureCRC"]),
    ("sm4", &["FeatureSM4"]),
    ("sha3", &["FeatureSHA3"]),
    ("sha2", &["FeatureSHA2"]),
    ("aes", &["FeatureAES"]),
    ("crypto", &["FeatureCrypto"]),
    ("fp", &["FeatureFPARMv8"]),
    ("simd", &["FeatureNEON"]),
    ("ras", &["FeatureRAS"]),
    ("rasv2", &["FeatureRASv2"]),
    ("lse", &["FeatureLSE"]),
    ("predres", &["FeaturePredRes"]),
    ("predres2", &["FeatureSPECRES2"]),
    ("ccdp", &["FeatureCacheDeepPersist"]),
    ("mte", &["FeatureMTE"]),
    ("memtag", &["FeatureMTE"]),
    ("tlb-rmi", &["FeatureTLB_RMI"]),
    ("pan", &["FeaturePAN"]),
    ("pan-rwv", &["FeaturePAN_RWV"]),
    ("ccpp", &["FeatureCCPP"]),
    ("rcpc", &["FeatureRCPC"]),
    ("rng", &["FeatureRandGen"]),
    ("sve", &["FeatureSVE"]),
    ("sve-b16b16", &["FeatureSVEB16B16"]),
    ("sve2", &["FeatureSVE2"]),
    ("sve-aes", &["FeatureSVEAES"]),
    ("sve2-aes", &["FeatureAliasSVE2AES", "FeatureSVEAES"]),
    ("sve-sm4", &["FeatureSVESM4"]),
    ("sve2-sm4", &["FeatureAliasSVE2SM4", "FeatureSVESM4"]),
    ("sve-sha3", &["FeatureSVESHA3"]),
    ("sve2-sha3", &["FeatureAliasSVE2SHA3", "FeatureSVESHA3"]),
    ("sve-bitperm", &["FeatureSVEBitPerm"]),
    (
        "sve2-bitperm",
        &[
            "FeatureAliasSVE2BitPerm",
            "FeatureSVEBitPerm",
            "FeatureSVE2",
        ],
    ),
    ("sve2p1", &["FeatureSVE2p1"]),
    ("ls64", &["FeatureLS64"]),
    ("xs", &["FeatureXS"]),
    ("pauth", &["FeaturePAuth"]),
    ("flagm", &["FeatureFlagM"]),
    ("rme", &["FeatureRME"]),
    ("sme", &["FeatureSME"]),
    ("sme-f64f64", &["FeatureSMEF64F64"]),
    ("sme-f16f16", &["FeatureSMEF16F16"]),
    ("sme-i16i64", &["FeatureSMEI16I64"]),
    ("sme2", &["FeatureSME2"]),
    ("sme2p1", &["FeatureSME2p1"]),
    ("sme-b16b16", &["FeatureSMEB16B16"]),
    ("hbc", &["FeatureHBC"]),
    ("mops", &["FeatureMOPS"]),
    ("mec", &["FeatureMEC"]),
    ("the", &["FeatureTHE"]),
    ("d128", &["FeatureD128"]),
    ("lse128", &["FeatureLSE128"]),
    ("ite", &["FeatureITE"]),
    ("cssc", &["FeatureCSSC"]),
    ("rcpc3", &["FeatureRCPC3"]),
    ("gcs", &["FeatureGCS"]),
    ("bf16", &["FeatureBF16"]),
    ("compnum", &["FeatureComplxNum"]),
    ("dotprod", &["FeatureDotProd"]),
    ("f32mm", &["FeatureMatMulFP32"]),
    ("f64mm", &["FeatureMatMulFP64"]),
    ("fp16", &["FeatureFullFP16"]),
    ("fp16fml", &["FeatureFP16FML"]),
    ("i8mm", &["FeatureMatMulInt8"]),
    ("lor", &["FeatureLOR"]),
    ("profile", &["FeatureSPE"]),
    ("rdm", &["FeatureRDM"]),
    ("rdma", &["FeatureRDM"]),
    ("sb", &["FeatureSB"]),
    ("ssbs", &["FeatureSSBS"]),
    ("fp8", &["FeatureFP8"]),
    ("faminmax", &["FeatureFAMINMAX"]),
    ("fp8fma", &["FeatureFP8FMA"]),
    ("ssve-fp8fma", &["FeatureSSVE_FP8FMA"]),
    ("fp8dot2", &["FeatureFP8DOT2"]),
    ("ssve-fp8dot2", &["FeatureSSVE_FP8DOT2"]),
    ("fp8dot4", &["FeatureFP8DOT4"]),
    ("ssve-fp8dot4", &["FeatureSSVE_FP8DOT4"]),
    ("lut", &["FeatureLUT"]),
    ("sme-lutv2", &["FeatureSME_LUTv2"]),
    ("sme-f8f16", &["FeatureSMEF8F16"]),
    ("sme-f8f32", &["FeatureSMEF8F32"]),
    ("sme-fa64", &["FeatureSMEFA64"]),
    ("cpa", &["FeatureCPA"]),
    ("tlbiw", &["FeatureTLBIW"]),
    ("pops", &["FeaturePoPS"]),
    ("cmpbr", &["FeatureCMPBR"]),
    ("f8f32mm", &["FeatureF8F32MM"]),
    ("f8f16mm", &["FeatureF8F16MM"]),
    ("fprcvt", &["FeatureFPRCVT"]),
    ("lsfe", &["FeatureLSFE"]),
    ("sme2p2", &["FeatureSME2p2"]),
    ("ssve-aes", &["FeatureSSVE_AES"]),
    ("sve2p2", &["FeatureSVE2p2"]),
    ("sve-aes2", &["FeatureSVEAES2"]),
    ("sve-bfscale", &["FeatureSVEBFSCALE"]),
    ("sve-f16f32mm", &["FeatureSVE_F16F32MM"]),
    ("lsui", &["FeatureLSUI"]),
    ("occmo", &["FeatureOCCMO"]),
    ("ssve-bitperm", &["FeatureSSVE_BitPerm"]),
    ("sme-mop4", &["FeatureSME_MOP4"]),
    ("sme-tmop", &["FeatureSME_TMOP"]),
    ("lscp", &["FeatureLSCP"]),
    ("tlbid", &["FeatureTLBID"]),
    ("mtetc", &["FeatureMTETC"]),
    ("gcie", &["FeatureGCIE"]),
    ("sme2p3", &["FeatureSME2p3"]),
    ("sve2p3", &["FeatureSVE2p3"]),
    ("sve-b16mm", &["FeatureSVE_B16MM"]),
    ("f16mm", &["FeatureF16MM"]),
    ("f16f32dot", &["FeatureF16F32DOT"]),
    ("f16f32mm", &["FeatureF16F32MM"]),
    ("mops-go", &["FeatureMOPS_GO"]),
    ("poe2", &["FeatureS1POE2"]),
    ("tev", &["FeatureTEV"]),
    ("btie", &["FeatureBTIE"]),
    ("hinte", &["FeatureHINTE"]),
    ("dit", &["FeatureDIT"]),
    ("brbe", &["FeatureBRBE"]),
    ("bti", &["FeatureBranchTargetId"]),
    ("fcma", &["FeatureComplxNum"]),
    ("jscvt", &["FeatureJS"]),
    ("pauth-lr", &["FeaturePAuthLR"]),
    ("ssve-fexpa", &["FeatureSSVE_FEXPA"]),
    ("wfxt", &["FeatureWFxT"]),
];

fn required_features_text(_target: &Target, requires: &[String]) -> String {
    for (feature, text) in [
        ("HasV8_0aOps", "ARMv8a"),
        ("HasV8_1aOps", "ARMv8.1a"),
        ("HasV8_2aOps", "ARMv8.2a"),
        ("HasV8_3aOps", "ARMv8.3a"),
        ("HasV8_4aOps", "ARMv8.4a"),
        ("HasV8_5aOps", "ARMv8.5a"),
        ("HasV8_6aOps", "ARMv8.6a"),
        ("HasV8_7aOps", "ARMv8.7a"),
        ("HasV8_8aOps", "ARMv8.8a"),
        ("HasV8_9aOps", "ARMv8.9a"),
        ("HasV9_0aOps", "ARMv9-a"),
        ("HasV9_1aOps", "ARMv9.1a"),
        ("HasV9_2aOps", "ARMv9.2a"),
        ("HasV9_3aOps", "ARMv9.3a"),
        ("HasV9_4aOps", "ARMv9.4a"),
        ("HasV8_0rOps", "ARMv8r"),
    ] {
        if requires.iter().any(|held| held == feature) {
            return text.to_string();
        }
    }
    let names: Vec<&str> = EXTENSION_DEFS
        .iter()
        .filter(|(_, features)| {
            features
                .iter()
                .any(|feature| requires.iter().any(|held| held == feature))
        })
        .map(|(name, _)| *name)
        .collect();
    if names.is_empty() {
        "(unknown)".to_string()
    } else {
        names.join(", ")
    }
}

impl<'s> Assembler<'s> {
    fn arm_sys_cr_operands(
        &mut self,
        operands: &mut Vec<ParsedOperand>,
        encoding: u32,
        start: usize,
    ) {
        let op2 = i64::from(encoding & 7);
        let cm = (encoding & 0x78) >> 3;
        let cn = (encoding & 0x780) >> 7;
        let op1 = i64::from((encoding & 0x3800) >> 11);
        for operand in [
            ArmOperand::Immediate(Expression::Constant(op1)),
            ArmOperand::SystemCr(cn),
            ArmOperand::SystemCr(cm),
            ArmOperand::Immediate(Expression::Constant(op2)),
        ] {
            operands.push(ParsedOperand {
                kind: ParsedKind::Arm(Box::new(operand)),
                start,
                end: start,
            });
        }
    }

    fn arm_parse_sys_alias(
        &mut self,
        head: &str,
        name_start: usize,
    ) -> Result<Vec<ParsedOperand>, ()> {
        let mut operands = Vec::new();
        self.arm_mnemonic = head.to_string();
        self.arm_push_token(&mut operands, "sys", name_start);
        let start = self.lexer.token.start;
        let op = self.lexer.token.text.clone();
        let all = self.arm_features.contains("FeatureAll");
        let tlbid = self.arm_features.contains("FeatureTLBID");
        let mut expect_register = true;
        let mut optional_register = false;
        let tables = system_tables_for();
        let group = match head {
            "ic" => Some("IC"),
            "dc" => Some("DC"),
            "at" => Some("AT"),
            "tlbi" => Some("TLBI"),
            "gic" => Some("GIC"),
            "gsb" => Some("GSB"),
            "plbi" => Some("PLBI"),
            _ => None,
        };
        if let Some(group) = group {
            let found = tables.sys_aliases.get(group).and_then(|list| {
                list.iter()
                    .find(|entry| entry.name.eq_ignore_ascii_case(&op))
            });
            let Some(entry) = found else {
                self.token_error(&format!(
                    "invalid operand for {} instruction",
                    head.to_ascii_uppercase()
                ));
                return Err(());
            };
            if !predicates_features(&entry.requires, &self.arm_features) {
                let text = format!(
                    "{} {} requires: {}",
                    head.to_ascii_uppercase(),
                    entry.name,
                    required_features_text(self.target, &entry.requires)
                );
                self.token_error(&text);
                return Err(());
            }
            if matches!(group, "IC" | "TLBI" | "GIC" | "GSB" | "PLBI") {
                expect_register = entry.needs_register;
            }
            if matches!(group, "TLBI" | "PLBI") && (all || tlbid) {
                optional_register = entry.optional_register;
            }
            self.arm_sys_cr_operands(&mut operands, entry.encoding, start);
        } else {
            if !op.eq_ignore_ascii_case("rctx") {
                self.token_error("invalid operand for prediction restriction instruction");
                return Err(());
            }
            let predres = all || self.arm_features.contains("FeaturePredRes");
            let specres2 = all || self.arm_features.contains("FeatureSPECRES2");
            if head == "cosp" && !specres2 {
                self.token_error("COSP requires: predres2");
                return Err(());
            }
            if !predres {
                self.token_error(&format!(
                    "{}RCTX requires: predres",
                    head.to_ascii_uppercase()
                ));
                return Err(());
            }
            let op2 = match head {
                "cfp" => 0b100,
                "dvp" => 0b101,
                "cosp" => 0b110,
                _ => 0b111,
            };
            let encoding = (0b011_0111_0011u32 << 3) | op2;
            self.arm_sys_cr_operands(&mut operands, encoding, start);
        }
        self.lex();
        let mut has_register = false;
        if self.parse_optional_token(TokenKind::Comma) {
            if !self.is(TokenKind::Identifier) || self.arm_parse_register(&mut operands) {
                self.token_error("expected register operand");
                return Err(());
            }
            has_register = true;
        }
        if !optional_register {
            if expect_register && !has_register {
                self.token_error(&format!("specified {head} op requires a register"));
                return Err(());
            } else if !expect_register && has_register {
                self.token_error(&format!("specified {head} op does not use a register"));
                return Err(());
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

impl<'s> Assembler<'s> {
    fn arm_parse_sysl_alias(
        &mut self,
        head: &str,
        name_start: usize,
    ) -> Result<Vec<ParsedOperand>, ()> {
        let mut operands = Vec::new();
        self.arm_mnemonic = head.to_string();
        self.arm_push_token(&mut operands, "sysl", name_start);
        if !self.is(TokenKind::Identifier) || self.arm_parse_register(&mut operands) {
            self.token_error("expected register operand");
            return Err(());
        }
        if self.parse_token(TokenKind::Comma, "unexpected token") {
            return Err(());
        }
        let start = self.lexer.token.start;
        let location = self.location();
        let op = self.lexer.token.text.clone();
        self.lex();
        let tables = system_tables_for();
        let found = tables.sys_aliases.get("GICR").and_then(|list| {
            list.iter()
                .find(|entry| entry.name.eq_ignore_ascii_case(&op))
        });
        let Some(entry) = found else {
            self.error(location, "invalid operand for GICR instruction");
            return Err(());
        };
        if !predicates_features(&entry.requires, &self.arm_features) {
            let text = format!(
                "GICR {} requires: {}",
                entry.name,
                required_features_text(self.target, &entry.requires)
            );
            self.error(location, &text);
            return Err(());
        }
        self.arm_sys_cr_operands(&mut operands, entry.encoding, start);
        if self.parse_token(
            TokenKind::EndOfStatement,
            "unexpected token in argument list",
        ) {
            return Err(());
        }
        Ok(operands)
    }

    fn arm_parse_sysp_alias(
        &mut self,
        head: &str,
        name_start: usize,
    ) -> Result<Vec<ParsedOperand>, ()> {
        let mut operands = Vec::new();
        self.arm_mnemonic = head.to_string();
        self.arm_push_token(&mut operands, "sysp", name_start);
        let start = self.lexer.token.start;
        let op = self.lexer.token.text.clone();
        let tables = system_tables_for();
        let found = tables.sys_aliases.get("TLBIP").and_then(|list| {
            list.iter()
                .find(|entry| entry.name.eq_ignore_ascii_case(&op))
        });
        let Some(entry) = found else {
            self.token_error("invalid operand for TLBIP instruction");
            return Err(());
        };
        let permitted = self.arm_features.contains("FeatureAll")
            || self.arm_features.contains("FeatureD128")
            || (entry.allow_tlbid && self.arm_features.contains("FeatureTLBID"));
        if !predicates_features(&entry.requires, &self.arm_features) || !permitted {
            let text = if entry.allow_tlbid {
                "instruction requires: tlbid or d128"
            } else {
                "instruction requires: d128"
            };
            self.token_error(text);
            return Err(());
        }
        self.arm_sys_cr_operands(&mut operands, entry.encoding, start);
        self.lex();
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return Err(());
        }
        if !self.is(TokenKind::Identifier) {
            self.token_error("expected register identifier");
            return Err(());
        }
        let mut status = self.arm_try_parse_syp_xzr_pair(&mut operands);
        if status == Status::NoMatch {
            status = self.arm_try_parse_gpr_seq_pair(&mut operands);
        }
        if status != Status::Success {
            self.token_error(&format!("specified {head} op requires a pair of registers"));
            return Err(());
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

impl<'t> ArmPrinter<'t> {
    fn system_encoding(
        &self,
        instruction: &MachineInstruction,
        first: usize,
    ) -> Option<(u32, u32, u32)> {
        let value = |index: usize| match instruction.operand(index) {
            MachineOperand::Immediate(value) => Some(*value as u32),
            _ => None,
        };
        let (op1, cn, cm, op2) = (
            value(first)?,
            value(first + 1)?,
            value(first + 2)?,
            value(first + 3)?,
        );
        Some((cn, cm, op2 | (cm << 3) | (cn << 7) | (op1 << 11)))
    }

    fn range_prefetch_alias(
        &self,
        instruction: &MachineInstruction,
        extended: bool,
    ) -> Option<String> {
        let operation = instruction.immediate_at(0) as u32;
        if operation & 0x18 != 0x18 {
            return None;
        }
        let mut register = self.register_text(instruction, 2);
        if register.starts_with('w') {
            register.replace_range(0..1, "x");
        }
        let sign = instruction.immediate_at(3) as u32 & 1;
        let shift = instruction.immediate_at(4) as u32 & 1;
        let option = u32::from(extended);
        let code = (sign << 5) | (option << 4) | (shift << 3) | (operation & 7);
        let tables = system_tables_for();
        let name = match tables.range_prefetch_names.get(&code) {
            Some(name) => name.clone(),
            None => format!("#{}", format_immediate(i64::from(code))),
        };
        Some(format!(
            "\trprfm {name}, {register}, [{}]",
            self.register_text(instruction, 1)
        ))
    }

    fn sysl_alias(&self, instruction: &MachineInstruction) -> Option<String> {
        let (cn, cm, encoding) = self.system_encoding(instruction, 1)?;
        if cn != 12 || cm != 3 {
            return None;
        }
        let tables = system_tables_for();
        let entry = tables
            .sys_aliases
            .get("GICR")?
            .iter()
            .filter(|entry| {
                entry.encoding == encoding && predicates_features(&entry.requires, self.features)
            })
            .min_by_key(|entry| entry.order)?;
        Some(format!(
            "\tgicr\t{}, {}",
            self.register_text(instruction, 0),
            entry.name.to_ascii_lowercase()
        ))
    }

    fn sysp_alias(&self, instruction: &MachineInstruction) -> Option<String> {
        let (cn, _, encoding) = self.system_encoding(instruction, 0)?;
        if cn != 8 && cn != 9 {
            return None;
        }
        let tables = system_tables_for();
        let entry = tables
            .sys_aliases
            .get("TLBIP")?
            .iter()
            .filter(|entry| {
                entry.encoding == encoding
                    && predicates_features(&entry.requires, self.features)
                    && (self.features.contains("FeatureAll")
                        || self.features.contains("FeatureD128")
                        || (entry.allow_tlbid && self.features.contains("FeatureTLBID")))
            })
            .min_by_key(|entry| entry.order)?;
        let pair = if self.register_text(instruction, 4) == "xzr" {
            "xzr, xzr".to_string()
        } else {
            match instruction.operand(4) {
                MachineOperand::Register(register) => self.target.registers[*register as usize]
                    .sub_registers
                    .iter()
                    .map(|member| self.register_name(*member))
                    .collect::<Vec<String>>()
                    .join(", "),
                _ => return None,
            }
        };
        Some(format!(
            "\ttlbip\t{}, {pair}",
            entry.name.to_ascii_lowercase()
        ))
    }

    fn sys_alias(&self, instruction: &MachineInstruction) -> Option<String> {
        let value = |index: usize| match instruction.operand(index) {
            MachineOperand::Immediate(value) => Some(*value as u32),
            _ => None,
        };
        let (op1, cn, cm, op2) = (value(0)?, value(1)?, value(2)?, value(3)?);
        let encoding = op2 | (cm << 3) | (cn << 7) | (op1 << 11);
        let tables = system_tables_for();
        let all = self.features.contains("FeatureAll");
        let tlbid = self.features.contains("FeatureTLBID");
        let find = |group: &str| -> Option<&SysAlias> {
            tables
                .sys_aliases
                .get(group)?
                .iter()
                .filter(|entry| {
                    entry.encoding == encoding
                        && predicates_features(&entry.requires, self.features)
                })
                .min_by_key(|entry| entry.order)
        };
        let needs_register;
        let mut optional_register = false;
        let instruction_name;
        let name;
        if cn == 7 {
            match cm {
                1 | 3 if op1 == 3 && cm == 3 || (cm == 1 && op1 == 3) => {
                    if op1 != 3 || cn != 7 || cm != 3 {
                        return None;
                    }
                    let required = if op2 == 6 {
                        "FeatureSPECRES2"
                    } else {
                        "FeaturePredRes"
                    };
                    if !(all || self.features.contains(required)) {
                        return None;
                    }
                    needs_register = true;
                    instruction_name = match op2 {
                        4 => "cfp",
                        5 => "dvp",
                        6 => "cosp",
                        7 => "cpp",
                        _ => return None,
                    };
                    name = "rctx".to_string();
                }
                1 if op1 == 0 => {
                    let entry = find("IC")?;
                    needs_register = entry.needs_register;
                    instruction_name = "ic";
                    name = entry.name.clone();
                }
                5 => {
                    let entry = find("IC")?;
                    needs_register = entry.needs_register;
                    instruction_name = "ic";
                    name = entry.name.clone();
                }
                4 | 6 | 10..=14 => {
                    let entry = find("DC")?;
                    needs_register = true;
                    instruction_name = "dc";
                    name = entry.name.clone();
                }
                8 | 9 => {
                    let entry = find("AT")?;
                    needs_register = true;
                    instruction_name = "at";
                    name = entry.name.clone();
                }
                15 => {
                    if let Some(entry) = find("AT") {
                        needs_register = true;
                        instruction_name = "at";
                        name = entry.name.clone();
                    } else {
                        let entry = find("DC")?;
                        needs_register = true;
                        instruction_name = "dc";
                        name = entry.name.clone();
                    }
                }
                _ => return None,
            }
        } else if cn == 8 || cn == 9 {
            let entry = find("TLBI")?;
            needs_register = entry.needs_register;
            if all || tlbid {
                optional_register = entry.optional_register;
            }
            instruction_name = "tlbi";
            name = entry.name.clone();
        } else if cn == 12 {
            if cm != 0 {
                let entry = find("GIC")?;
                needs_register = entry.needs_register;
                instruction_name = "gic";
                name = entry.name.clone();
            } else {
                let entry = find("GSB")?;
                needs_register = false;
                instruction_name = "gsb";
                name = entry.name.clone();
            }
        } else if cn == 10 {
            let entry = find("PLBI")?;
            needs_register = entry.needs_register;
            if all || tlbid {
                optional_register = entry.optional_register;
            }
            instruction_name = "plbi";
            name = entry.name.clone();
        } else {
            return None;
        }
        let register = self.register_text(instruction, 4);
        let not_zero = register != "xzr";
        if not_zero && !needs_register && !optional_register {
            return None;
        }
        let mut text = format!("\t{instruction_name}\t{name}").to_ascii_lowercase();
        if needs_register || (optional_register && not_zero) {
            text.push_str(&format!(", {register}"));
        }
        Some(text)
    }
}

pub mod native {
    use super::*;
    use crate::compilers::compiler_infrastructure::{
        AtomicOp, BinaryOp, BlockCall, Comparison, Conversion, FloatComparison, Function,
        FunctionId, Instruction, Linkage, Module, Operation, Signature, Terminator, Type, UnaryOp,
        Value,
    };
    use crate::compilers::x86_64::codegen::{
        self as shared, Emitter, ARRAY_HEADER, ARRAY_LENGTH_OFFSET, TRAP_ABSTRACT_METHOD,
        TRAP_CONVERSION_RANGE, TRAP_DIVISION_BY_ZERO, TRAP_EXPLICIT, TRAP_INDEX_OUT_OF_BOUNDS,
        TRAP_MISSING_IMPLEMENTATION, TRAP_NULL_REFERENCE, TRAP_OVERLAPPING_COPY, TRAP_UNREACHABLE,
    };
    use std::ops::{Deref, DerefMut};

    const TRIPLE: &str = "aarch64-unknown-linux-gnu";
    const A: &str = "x9";
    const B: &str = "x10";
    const C: &str = "x11";
    const D: &str = "x12";
    const E: &str = "x13";
    const F: &str = "x14";

    fn narrow(register: &str) -> String {
        format!("w{}", &register[1..])
    }

    fn float_register(ty: Type, number: usize) -> String {
        format!("{}{number}", if shared::single(ty) { 's' } else { 'd' })
    }

    #[derive(Clone, Copy)]
    enum Place {
        Integer(usize),
        Float(usize),
        Stack(usize),
    }

    fn classify(types: &[Type]) -> (Vec<Place>, usize) {
        let mut integers = 0;
        let mut floats = 0;
        let mut stack = 0;
        let mut places = Vec::with_capacity(types.len());
        for ty in types {
            if shared::is_float(*ty) {
                if floats < 8 {
                    places.push(Place::Float(floats));
                    floats += 1;
                } else {
                    places.push(Place::Stack(stack));
                    stack += 1;
                }
            } else if integers < 8 {
                places.push(Place::Integer(integers));
                integers += 1;
            } else {
                places.push(Place::Stack(stack));
                stack += 1;
            }
        }
        (places, stack)
    }

    struct Frame {
        function: FunctionId,
        slots: BTreeMap<usize, i64>,
    }

    struct Arm<'m> {
        base: Emitter<'m>,
    }

    impl<'m> Deref for Arm<'m> {
        type Target = Emitter<'m>;

        fn deref(&self) -> &Self::Target {
            &self.base
        }
    }

    impl DerefMut for Arm<'_> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.base
        }
    }

    impl<'m> Arm<'m> {
        fn slot(value: Value) -> i64 {
            -16 * (i64::from(value.0) + 1)
        }

        fn access(&mut self, load: bool, register: &str, value: Value) {
            let at = Self::slot(value);
            if (-256..=255).contains(&at) {
                let mnemonic = if load { "ldur" } else { "stur" };
                self.line(&format!("{mnemonic} {register}, [x29, #{at}]"));
            } else {
                self.constant(at as u64, "x16");
                let mnemonic = if load { "ldr" } else { "str" };
                self.line(&format!("{mnemonic} {register}, [x29, x16]"));
            }
        }

        fn load(&mut self, value: Value, register: &str) {
            self.access(true, register, value);
        }

        fn store(&mut self, register: &str, value: Value) {
            self.access(false, register, value);
        }

        fn store_result(&mut self, register: &str, result: Option<Value>) {
            if let Some(result) = result {
                self.store(register, result);
            }
        }

        fn load_signed(&mut self, value: Value, register: &str, bits: u32) {
            self.load(value, register);
            self.sign_extend_register(register, bits);
        }

        fn load_float(&mut self, value: Value, ty: Type, number: usize) {
            let register = float_register(ty, number);
            self.access(true, &register, value);
        }

        fn store_float(&mut self, number: usize, value: Value, ty: Type) {
            let register = float_register(ty, number);
            self.access(false, &register, value);
        }

        fn mask(&mut self, register: &str, bits: u32) {
            let low = narrow(register);
            match bits {
                1 => self.line(&format!("and {low}, {low}, #1")),
                8 => self.line(&format!("uxtb {low}, {low}")),
                16 => self.line(&format!("uxth {low}, {low}")),
                32 => self.line(&format!("mov {low}, {low}")),
                _ => {}
            }
        }

        fn sign_extend_register(&mut self, register: &str, bits: u32) {
            let low = narrow(register);
            match bits {
                1 => {
                    self.line(&format!("and {register}, {register}, #1"));
                    self.line(&format!("neg {register}, {register}"));
                }
                8 => self.line(&format!("sxtb {register}, {low}")),
                16 => self.line(&format!("sxth {register}, {low}")),
                32 => self.line(&format!("sxtw {register}, {low}")),
                _ => {}
            }
        }

        fn constant(&mut self, value: u64, register: &str) {
            let chunk = |value: u64, at: u32| (value >> (16 * at)) & 0xffff;
            let zeros = (0..4).filter(|at| chunk(value, *at) == 0).count();
            let ones = (0..4).filter(|at| chunk(value, *at) == 0xffff).count();
            let shift = |at: u32| {
                if at == 0 {
                    String::new()
                } else {
                    format!(", lsl #{}", 16 * at)
                }
            };
            if ones > zeros {
                let inverse = !value;
                let first = (0..4).find(|at| chunk(inverse, *at) != 0).unwrap_or(0);
                self.line(&format!(
                    "movn {register}, #{}{}",
                    chunk(inverse, first),
                    shift(first)
                ));
                for at in (0..4).filter(|at| *at != first) {
                    if chunk(value, at) != 0xffff {
                        self.line(&format!(
                            "movk {register}, #{}{}",
                            chunk(value, at),
                            shift(at)
                        ));
                    }
                }
            } else {
                let first = (0..4).find(|at| chunk(value, *at) != 0).unwrap_or(0);
                self.line(&format!(
                    "movz {register}, #{}{}",
                    chunk(value, first),
                    shift(first)
                ));
                for at in (0..4).filter(|at| *at > first) {
                    if chunk(value, at) != 0 {
                        self.line(&format!(
                            "movk {register}, #{}{}",
                            chunk(value, at),
                            shift(at)
                        ));
                    }
                }
            }
        }

        fn adjust(&mut self, destination: &str, source: &str, amount: i64) {
            if amount == 0 {
                if destination != source {
                    self.line(&format!("mov {destination}, {source}"));
                }
            } else if (0..4096).contains(&amount) {
                self.line(&format!("add {destination}, {source}, #{amount}"));
            } else if (-4095..0).contains(&amount) {
                self.line(&format!("sub {destination}, {source}, #{}", -amount));
            } else {
                self.constant(amount as u64, "x16");
                self.line(&format!("add {destination}, {source}, x16"));
            }
        }

        fn global_address(&mut self, symbol: &str, register: &str) {
            self.line(&format!("adrp {register}, {symbol}"));
            self.line(&format!("add {register}, {register}, :lo12:{symbol}"));
        }

        fn pending(&mut self, load: bool, register: &str) {
            self.line("adrp x16, __omni_pending");
            let mnemonic = if load { "ldr" } else { "str" };
            self.line(&format!(
                "{mnemonic} {register}, [x16, :lo12:__omni_pending]"
            ));
        }

        fn memory(
            &mut self,
            load: bool,
            suffix: &str,
            register: &str,
            address: &str,
            offset: i64,
            size: i64,
        ) {
            let plain = if load { "ldr" } else { "str" };
            let unscaled = if load { "ldur" } else { "stur" };
            if offset >= 0 && offset % size == 0 && offset / size <= 4095 {
                if offset == 0 {
                    self.line(&format!("{plain}{suffix} {register}, [{address}]"));
                } else {
                    self.line(&format!(
                        "{plain}{suffix} {register}, [{address}, #{offset}]"
                    ));
                }
            } else if (-256..=255).contains(&offset) {
                self.line(&format!(
                    "{unscaled}{suffix} {register}, [{address}, #{offset}]"
                ));
            } else {
                self.constant(offset as u64, "x16");
                self.line(&format!("add x16, {address}, x16"));
                self.line(&format!("{plain}{suffix} {register}, [x16]"));
            }
        }

        fn memory_load(&mut self, ty: Type, address: &str, offset: i64, register: &str) {
            let size = self.layout.size_of(ty);
            let low = narrow(register);
            match size {
                1 => self.memory(true, "b", &low, address, offset, 1),
                2 => self.memory(true, "h", &low, address, offset, 2),
                4 => self.memory(true, "", &low, address, offset, 4),
                _ => self.memory(true, "", register, address, offset, 8),
            }
        }

        fn memory_store(&mut self, ty: Type, register: &str, address: &str, offset: i64) {
            let size = self.layout.size_of(ty);
            let low = narrow(register);
            match size {
                1 => self.memory(false, "b", &low, address, offset, 1),
                2 => self.memory(false, "h", &low, address, offset, 2),
                4 => self.memory(false, "", &low, address, offset, 4),
                _ => self.memory(false, "", register, address, offset, 8),
            }
        }

        fn trap(&mut self, code: i64) {
            self.constant(code as u64, "x0");
            self.line("mov x1, xzr");
            self.line("bl __omni_trap");
        }

        fn trap_unless_nonzero(&mut self, register: &str, code: i64) {
            let skip = self.fresh();
            self.line(&format!("cbnz {register}, {skip}"));
            self.trap(code);
            self.label(&skip);
        }

        fn nan_into(&mut self, ty: Type, number: usize) {
            if shared::single(ty) {
                self.line("movz w16, #0x7fc0, lsl #16");
                self.line(&format!("fmov s{number}, w16"));
            } else {
                self.line("movz x16, #0x7ff8, lsl #48");
                self.line(&format!("fmov d{number}, x16"));
            }
        }

        fn canonical_nan(&mut self, number: usize, ty: Type) {
            let skip = self.fresh();
            let register = float_register(ty, number);
            self.line(&format!("fcmp {register}, {register}"));
            self.line(&format!("b.vc {skip}"));
            self.nan_into(ty, number);
            self.label(&skip);
        }

        fn emit_module(&mut self) -> Result<(), Diagnostic> {
            shared::check_supported(self.module)?;
            for index in 0..self.module.functions.len() {
                let id = FunctionId(index as u32);
                if !self.module.functions[index].is_declaration() {
                    self.emit_function(id)?;
                }
            }
            self.emit_globals();
            self.emit_strings();
            self.emit_classes();
            Ok(())
        }

        fn emit_function(&mut self, id: FunctionId) -> Result<(), Diagnostic> {
            let module = self.module;
            let function = &module.functions[id.index()];
            let symbol = shared::function_symbol(module, id);
            let values = function.value_types.len() as i64;
            let mut base = 16 * values;
            let mut slots = BTreeMap::new();
            let mut outgoing = 0usize;
            let mut calls: Vec<&Instruction> = function.instructions.iter().collect();
            for block in &function.blocks {
                if let Terminator::Invoke { call, .. } = &block.terminator {
                    calls.push(call);
                }
            }
            for (index, instruction) in function.instructions.iter().enumerate() {
                if let Operation::StackSlot { size, align } = instruction.operation {
                    let align = i64::from(align.clamp(1, 16));
                    base = (base + i64::from(size.max(1)) + align - 1) / align * align;
                    slots.insert(index, -base);
                }
            }
            for call in &calls {
                let types: Vec<Type> = match &call.operation {
                    Operation::Call(_) => call
                        .arguments
                        .iter()
                        .map(|value| function.type_of(*value))
                        .collect(),
                    Operation::CallIndirect(_) => call.arguments[1..]
                        .iter()
                        .map(|value| function.type_of(*value))
                        .collect(),
                    _ => continue,
                };
                let (_, stack) = classify(&types);
                outgoing = outgoing.max(stack);
            }
            let frame_size = (base + 15) / 16 * 16 + (outgoing as i64 * 8 + 15) / 16 * 16;
            self.line(".text");
            self.line(".p2align 2");
            match function.linkage {
                Linkage::Export => self.line(&format!(".globl {symbol}")),
                Linkage::Weak => self.line(&format!(".weak {symbol}")),
                _ => {}
            }
            self.line(&format!(".type {symbol}, @function"));
            self.label(&symbol);
            self.line("stp x29, x30, [sp, #-16]!");
            self.line("mov x29, sp");
            if frame_size > 0 {
                self.stack_adjust(frame_size, false);
            }
            let parameters = function.blocks[0].parameters.clone();
            let types: Vec<Type> = parameters
                .iter()
                .map(|value| function.type_of(*value))
                .collect();
            let (places, _) = classify(&types);
            for ((parameter, ty), place) in parameters.iter().zip(&types).zip(&places) {
                match place {
                    Place::Integer(index) => self.store(&format!("x{index}"), *parameter),
                    Place::Float(index) => self.store_float(*index, *parameter, *ty),
                    Place::Stack(index) => {
                        self.memory(true, "", A, "x29", 16 + 8 * *index as i64, 8);
                        self.store(A, *parameter);
                    }
                }
            }
            let frame = Frame {
                function: id,
                slots,
            };
            for (index, block) in function.blocks.iter().enumerate() {
                self.label(&Emitter::block_label(id, index));
                for inst in &block.instructions {
                    let instruction = &function.instructions[inst.index()];
                    self.emit_instruction(&frame, function, inst.index(), instruction)?;
                }
                self.emit_terminator(&frame, function, &block.terminator)?;
            }
            self.label(&format!(".LF{}R", id.0));
            self.line("mov sp, x29");
            self.line("ldp x29, x30, [sp], #16");
            self.line("ret");
            self.label(&format!(".LF{}O", id.0));
            self.line("mov x0, xzr");
            self.line("mov sp, x29");
            self.line("ldp x29, x30, [sp], #16");
            self.line("ret");
            self.line(&format!(".size {symbol}, .-{symbol}"));
            Ok(())
        }

        fn stack_adjust(&mut self, amount: i64, grow: bool) {
            let mnemonic = if grow { "add" } else { "sub" };
            if amount < 4096 {
                self.line(&format!("{mnemonic} sp, sp, #{amount}"));
            } else {
                self.constant(amount as u64, "x16");
                self.line(&format!("{mnemonic} sp, sp, x16"));
            }
        }

        fn edge(&mut self, frame: &Frame, function: &Function, call: &BlockCall, prefix: bool) {
            let parameters = &function.blocks[call.block.index()].parameters;
            let mut incoming = 0;
            if prefix {
                self.line(&format!("str {A}, [sp, #-16]!"));
                incoming += 1;
            }
            for argument in &call.arguments {
                self.load(*argument, A);
                self.line(&format!("str {A}, [sp, #-16]!"));
                incoming += 1;
            }
            for parameter in parameters.iter().take(incoming).rev() {
                self.line(&format!("ldr {A}, [sp], #16"));
                self.store(A, *parameter);
            }
            for _ in parameters.len()..incoming {
                self.line("add sp, sp, #16");
            }
            let target = Emitter::block_label(frame.function, call.block.index());
            self.line(&format!("b {target}"));
        }

        fn may_throw(&self, instruction: &Instruction) -> bool {
            match &instruction.operation {
                Operation::Call(callee) => !self.module.functions[callee.index()].is_declaration(),
                Operation::CallIndirect(_) => true,
                _ => false,
            }
        }

        fn emit_call(
            &mut self,
            function: &Function,
            instruction: &Instruction,
        ) -> Result<(), Diagnostic> {
            let (arguments, _signature, target): (&[Value], Signature, Option<FunctionId>) =
                match &instruction.operation {
                    Operation::Call(callee) => (
                        &instruction.arguments,
                        self.module.functions[callee.index()].signature.clone(),
                        Some(*callee),
                    ),
                    Operation::CallIndirect(signature) => {
                        (&instruction.arguments[1..], signature.clone(), None)
                    }
                    _ => return Ok(()),
                };
            if let Some(callee) = target {
                let name = self.module.functions[callee.index()].name.as_str();
                if self.module.functions[callee.index()].is_declaration() {
                    match name {
                        shared::INTRINSIC_SYSCALL => {
                            for (at, argument) in arguments.iter().take(7).enumerate() {
                                let register = if at == 0 {
                                    "x8".to_string()
                                } else {
                                    format!("x{}", at - 1)
                                };
                                self.load(*argument, &register);
                            }
                            self.line("svc #0");
                            self.store_result("x0", instruction.result);
                            return Ok(());
                        }
                        shared::INTRINSIC_ADDRESS | shared::INTRINSIC_REFERENCE => {
                            self.load(arguments[0], A);
                            self.store_result(A, instruction.result);
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            let types: Vec<Type> = arguments
                .iter()
                .map(|value| function.type_of(*value))
                .collect();
            let (places, _) = classify(&types);
            for (argument, place) in arguments.iter().zip(&places) {
                if let Place::Stack(index) = place {
                    self.load(*argument, A);
                    self.memory(false, "", A, "sp", 8 * *index as i64, 8);
                }
            }
            if target.is_none() {
                self.load(instruction.arguments[0], "x17");
            }
            for ((argument, place), ty) in arguments.iter().zip(&places).zip(&types) {
                match place {
                    Place::Integer(index) => self.load(*argument, &format!("x{index}")),
                    Place::Float(index) => self.load_float(*argument, *ty, *index),
                    Place::Stack(_) => {}
                }
            }
            match target {
                Some(callee) => {
                    let symbol = shared::function_symbol(self.module, callee);
                    self.line(&format!("bl {symbol}"));
                }
                None => self.line("blr x17"),
            }
            if let Some(result) = instruction.result {
                let ty = function.type_of(result);
                if shared::is_float(ty) {
                    self.store_float(0, result, ty);
                } else {
                    self.mask("x0", shared::type_bits(ty));
                    self.store("x0", result);
                }
            }
            Ok(())
        }

        fn emit_terminator(
            &mut self,
            frame: &Frame,
            function: &Function,
            terminator: &Terminator,
        ) -> Result<(), Diagnostic> {
            match terminator {
                Terminator::Jump(call) => self.edge(frame, function, call, false),
                Terminator::Branch {
                    condition,
                    taken,
                    otherwise,
                } => {
                    let label = self.fresh();
                    self.load(*condition, A);
                    self.line(&format!("cbnz {A}, {label}"));
                    self.edge(frame, function, otherwise, false);
                    self.label(&label);
                    self.edge(frame, function, taken, false);
                }
                Terminator::Switch {
                    value,
                    cases,
                    default,
                } => {
                    let bits = shared::type_bits(function.type_of(*value));
                    self.load(*value, A);
                    let mut targets = Vec::new();
                    for (case, _) in cases {
                        let label = self.fresh();
                        self.constant((*case as u64) & shared::all_ones(bits), B);
                        self.line(&format!("cmp {A}, {B}"));
                        self.line(&format!("b.eq {label}"));
                        targets.push(label);
                    }
                    self.edge(frame, function, default, false);
                    for ((_, call), label) in cases.iter().zip(targets) {
                        self.label(&label);
                        self.edge(frame, function, call, false);
                    }
                }
                Terminator::Return(value) => {
                    if let Some(value) = value {
                        let ty = function.type_of(*value);
                        if shared::is_float(ty) {
                            self.load_float(*value, ty, 0);
                        } else {
                            self.load(*value, "x0");
                        }
                    }
                    self.line(&format!("b .LF{}R", frame.function.0));
                }
                Terminator::Unreachable => self.trap(TRAP_UNREACHABLE),
                Terminator::Throw(value) => {
                    self.load(*value, A);
                    self.trap_unless_nonzero(A, TRAP_NULL_REFERENCE);
                    self.pending(false, A);
                    self.line(&format!("b .LF{}O", frame.function.0));
                }
                Terminator::Invoke {
                    call,
                    normal,
                    unwind,
                } => {
                    self.emit_call(function, call)?;
                    if self.may_throw(call) {
                        let caught = self.fresh();
                        self.pending(true, A);
                        self.line(&format!("cbnz {A}, {caught}"));
                        self.edge(frame, function, normal, false);
                        self.label(&caught);
                        self.pending(false, "xzr");
                        self.edge(frame, function, unwind, true);
                    } else {
                        self.edge(frame, function, normal, false);
                    }
                }
            }
            Ok(())
        }

        fn emit_instruction(
            &mut self,
            frame: &Frame,
            function: &Function,
            index: usize,
            instruction: &Instruction,
        ) -> Result<(), Diagnostic> {
            let arguments = &instruction.arguments;
            let result = instruction.result;
            let result_type = result.map(|value| function.type_of(value));
            let argument_type = |at: usize| function.type_of(arguments[at]);
            match &instruction.operation {
                Operation::IntegerConstant(value) => {
                    let bits = shared::type_bits(result_type.unwrap_or(Type::I64));
                    self.constant((*value as u64) & shared::all_ones(bits), A);
                    self.store_result(A, result);
                }
                Operation::FloatConstant(bits) => {
                    let ty = result_type.unwrap_or(Type::F64);
                    let raw = if shared::single(ty) {
                        bits & 0xffff_ffff
                    } else {
                        *bits
                    };
                    self.constant(raw, A);
                    self.store_result(A, result);
                }
                Operation::Null | Operation::Poison => {
                    self.line(&format!("mov {A}, xzr"));
                    self.store_result(A, result);
                }
                Operation::GlobalAddress(id) => {
                    let symbol = shared::global_symbol(self.module, id.index());
                    self.global_address(&symbol, A);
                    self.store_result(A, result);
                }
                Operation::FunctionAddress(id) => {
                    let symbol = shared::function_symbol(self.module, *id);
                    self.global_address(&symbol, A);
                    self.store_result(A, result);
                }
                Operation::StringConstant(index) => {
                    self.global_address(&format!("__omni_s{index}"), A);
                    self.store_result(A, result);
                }
                Operation::Binary(op) => {
                    let ty = argument_type(0);
                    if op.is_float() {
                        self.float_binary(*op, ty, arguments[0], arguments[1], result);
                    } else {
                        self.integer_binary(
                            *op,
                            shared::type_bits(ty),
                            arguments[0],
                            arguments[1],
                            result,
                        );
                    }
                }
                Operation::Unary(op) => {
                    let ty = argument_type(0);
                    if op.is_float() {
                        self.float_unary(*op, ty, arguments[0], result);
                    } else {
                        self.integer_unary(*op, shared::type_bits(ty), arguments[0], result);
                    }
                }
                Operation::Compare(comparison) => {
                    let bits = shared::type_bits(argument_type(0));
                    let signed = matches!(
                        comparison,
                        Comparison::LessSigned
                            | Comparison::LessEqualSigned
                            | Comparison::GreaterSigned
                            | Comparison::GreaterEqualSigned
                    );
                    if signed {
                        self.load_signed(arguments[0], A, bits);
                        self.load_signed(arguments[1], B, bits);
                    } else {
                        self.load(arguments[0], A);
                        self.load(arguments[1], B);
                    }
                    let condition = match comparison {
                        Comparison::Equal => "eq",
                        Comparison::NotEqual => "ne",
                        Comparison::LessSigned => "lt",
                        Comparison::LessEqualSigned => "le",
                        Comparison::GreaterSigned => "gt",
                        Comparison::GreaterEqualSigned => "ge",
                        Comparison::LessUnsigned => "lo",
                        Comparison::LessEqualUnsigned => "ls",
                        Comparison::GreaterUnsigned => "hi",
                        Comparison::GreaterEqualUnsigned => "hs",
                    };
                    self.line(&format!("cmp {A}, {B}"));
                    self.line(&format!("cset w9, {condition}"));
                    self.store_result(A, result);
                }
                Operation::FloatCompare(comparison) => {
                    self.float_compare(*comparison, argument_type(0), arguments, result);
                }
                Operation::Convert(conversion) => {
                    let to = result_type.unwrap_or(Type::I64);
                    self.convert(*conversion, arguments[0], argument_type(0), to, result);
                }
                Operation::Select => {
                    self.load(arguments[0], C);
                    self.load(arguments[1], A);
                    self.load(arguments[2], B);
                    self.line(&format!("cmp {C}, #0"));
                    self.line(&format!("csel {A}, {A}, {B}, ne"));
                    self.store_result(A, result);
                }
                Operation::Splat | Operation::ExtractLane(_) | Operation::InsertLane(_) => {
                    return Err(shared::unsupported("vector lanes"));
                }
                Operation::StackSlot { .. } => {
                    let at = frame.slots.get(&index).copied().unwrap_or(0);
                    self.adjust(A, "x29", at);
                    self.store_result(A, result);
                }
                Operation::Load { .. } => {
                    let ty = result_type.unwrap_or(Type::I64);
                    self.load(arguments[0], B);
                    self.memory_load(ty, B, 0, A);
                    self.store_result(A, result);
                }
                Operation::Store { .. } => {
                    let ty = argument_type(0);
                    self.load(arguments[0], A);
                    self.load(arguments[1], B);
                    self.memory_store(ty, A, B, 0);
                }
                Operation::Offset => {
                    let bits = shared::type_bits(argument_type(1));
                    self.load(arguments[0], A);
                    self.load_signed(arguments[1], B, bits);
                    self.line(&format!("add {A}, {A}, {B}"));
                    self.store_result(A, result);
                }
                Operation::FieldAddress(structure, field) => {
                    let (offsets, _, _) = self
                        .layout
                        .struct_layout(&self.module.structs[structure.index()]);
                    let offset = offsets.get(*field as usize).copied().unwrap_or(0);
                    self.load(arguments[0], A);
                    self.adjust(A, A, i64::from(offset));
                    self.store_result(A, result);
                }
                Operation::ElementAddress(ty) => {
                    let bits = shared::type_bits(argument_type(1));
                    let size = self.layout.size_of(*ty);
                    self.load(arguments[0], A);
                    self.load_signed(arguments[1], B, bits);
                    self.constant(u64::from(size), C);
                    self.line(&format!("madd {A}, {B}, {C}, {A}"));
                    self.store_result(A, result);
                }
                Operation::AtomicUpdate(op, _) => {
                    let ty = argument_type(1);
                    let bits = shared::type_bits(ty);
                    self.load(arguments[0], D);
                    self.memory_load(ty, D, 0, A);
                    let signed = matches!(op, AtomicOp::MinimumSigned | AtomicOp::MaximumSigned);
                    if signed {
                        self.load_signed(arguments[1], B, bits);
                        self.line(&format!("mov {C}, {A}"));
                        self.sign_extend_register(C, bits);
                    } else {
                        self.load(arguments[1], B);
                        self.line(&format!("mov {C}, {A}"));
                    }
                    match op {
                        AtomicOp::Exchange => self.line(&format!("mov {C}, {B}")),
                        AtomicOp::Add => self.line(&format!("add {C}, {C}, {B}")),
                        AtomicOp::Subtract => self.line(&format!("sub {C}, {C}, {B}")),
                        AtomicOp::And => self.line(&format!("and {C}, {C}, {B}")),
                        AtomicOp::Nand => {
                            self.line(&format!("and {C}, {C}, {B}"));
                            self.line(&format!("mvn {C}, {C}"));
                        }
                        AtomicOp::Or => self.line(&format!("orr {C}, {C}, {B}")),
                        AtomicOp::Xor => self.line(&format!("eor {C}, {C}, {B}")),
                        AtomicOp::MinimumSigned => {
                            self.line(&format!("cmp {C}, {B}"));
                            self.line(&format!("csel {C}, {C}, {B}, lt"));
                        }
                        AtomicOp::MaximumSigned => {
                            self.line(&format!("cmp {C}, {B}"));
                            self.line(&format!("csel {C}, {C}, {B}, gt"));
                        }
                        AtomicOp::MinimumUnsigned => {
                            self.line(&format!("cmp {C}, {B}"));
                            self.line(&format!("csel {C}, {C}, {B}, lo"));
                        }
                        AtomicOp::MaximumUnsigned => {
                            self.line(&format!("cmp {C}, {B}"));
                            self.line(&format!("csel {C}, {C}, {B}, hi"));
                        }
                    }
                    self.memory_store(ty, C, D, 0);
                    self.store_result(A, result);
                }
                Operation::CompareExchange(..) => {
                    let ty = argument_type(1);
                    let skip = self.fresh();
                    self.load(arguments[0], D);
                    self.load(arguments[1], B);
                    self.load(arguments[2], C);
                    self.memory_load(ty, D, 0, A);
                    self.line(&format!("cmp {A}, {B}"));
                    self.line(&format!("b.ne {skip}"));
                    self.memory_store(ty, C, D, 0);
                    self.label(&skip);
                    self.store_result(A, result);
                }
                Operation::Fence(_) => self.line("dmb ish"),
                Operation::MemoryCopy => {
                    let skip = self.fresh();
                    let fine = self.fresh();
                    let again = self.fresh();
                    self.load(arguments[0], D);
                    self.load(arguments[1], E);
                    self.load(arguments[2], B);
                    self.line(&format!("cbz {B}, {skip}"));
                    self.line(&format!("add {A}, {E}, {B}"));
                    self.line(&format!("cmp {D}, {A}"));
                    self.line(&format!("b.hs {fine}"));
                    self.line(&format!("add {A}, {D}, {B}"));
                    self.line(&format!("cmp {E}, {A}"));
                    self.line(&format!("b.hs {fine}"));
                    self.trap(TRAP_OVERLAPPING_COPY);
                    self.label(&fine);
                    self.label(&again);
                    self.line(&format!("ldrb w9, [{E}], #1"));
                    self.line(&format!("strb w9, [{D}], #1"));
                    self.line(&format!("subs {B}, {B}, #1"));
                    self.line(&format!("b.ne {again}"));
                    self.label(&skip);
                }
                Operation::MemoryMove => {
                    let forward = self.fresh();
                    let done = self.fresh();
                    let up = self.fresh();
                    let down = self.fresh();
                    self.load(arguments[0], D);
                    self.load(arguments[1], E);
                    self.load(arguments[2], B);
                    self.line(&format!("cbz {B}, {done}"));
                    self.line(&format!("cmp {D}, {E}"));
                    self.line(&format!("b.ls {forward}"));
                    self.line(&format!("add {A}, {E}, {B}"));
                    self.line(&format!("cmp {D}, {A}"));
                    self.line(&format!("b.hs {forward}"));
                    self.line(&format!("add {E}, {E}, {B}"));
                    self.line(&format!("add {D}, {D}, {B}"));
                    self.label(&down);
                    self.line(&format!("ldrb w9, [{E}, #-1]!"));
                    self.line(&format!("strb w9, [{D}, #-1]!"));
                    self.line(&format!("subs {B}, {B}, #1"));
                    self.line(&format!("b.ne {down}"));
                    self.line(&format!("b {done}"));
                    self.label(&forward);
                    self.label(&up);
                    self.line(&format!("ldrb w9, [{E}], #1"));
                    self.line(&format!("strb w9, [{D}], #1"));
                    self.line(&format!("subs {B}, {B}, #1"));
                    self.line(&format!("b.ne {up}"));
                    self.label(&done);
                }
                Operation::MemoryFill => {
                    let done = self.fresh();
                    let again = self.fresh();
                    self.load(arguments[0], D);
                    self.load(arguments[1], A);
                    self.load(arguments[2], B);
                    self.line(&format!("cbz {B}, {done}"));
                    self.label(&again);
                    self.line(&format!("strb w9, [{D}], #1"));
                    self.line(&format!("subs {B}, {B}, #1"));
                    self.line(&format!("b.ne {again}"));
                    self.label(&done);
                }
                Operation::Call(_) | Operation::CallIndirect(_) => {
                    self.emit_call(function, instruction)?;
                    if self.may_throw(instruction) {
                        self.pending(true, "x16");
                        self.line(&format!("cbnz x16, .LF{}O", frame.function.0));
                    }
                }
                Operation::Allocate(class) => {
                    let (_, size) = self.field_layout(*class);
                    self.constant(u64::from(size), "x0");
                    self.global_address(&format!("__omni_c{}", class.0), "x1");
                    self.line("bl __omni_allocate");
                    self.store_result("x0", result);
                }
                Operation::AllocateArray(class) => {
                    let element = self.module.classes[class.index()]
                        .element
                        .unwrap_or(Type::I8);
                    let size = self.layout.size_of(element);
                    let bits = shared::type_bits(argument_type(0));
                    self.load_signed(arguments[0], "x0", bits);
                    self.constant(u64::from(size), "x1");
                    self.global_address(&format!("__omni_c{}", class.0), "x2");
                    self.line("bl __omni_allocate_array");
                    self.store_result("x0", result);
                }
                Operation::LoadField(class, field) | Operation::StoreField(class, field) => {
                    let (offsets, _) = self.field_layout(*class);
                    let offset = i64::from(offsets.get(*field as usize).copied().unwrap_or(0));
                    let ty = self
                        .module
                        .field_types(*class)
                        .get(*field as usize)
                        .copied()
                        .unwrap_or(Type::I64);
                    self.load(arguments[0], B);
                    self.trap_unless_nonzero(B, TRAP_NULL_REFERENCE);
                    if matches!(instruction.operation, Operation::StoreField(..)) {
                        self.load(arguments[1], A);
                        self.memory_store(ty, A, B, offset);
                    } else {
                        self.memory_load(ty, B, offset, A);
                        self.store_result(A, result);
                    }
                }
                Operation::LoadElement(ty) | Operation::StoreElement(ty) => {
                    let bits = shared::type_bits(argument_type(1));
                    let size = u64::from(self.layout.size_of(*ty));
                    let fine = self.fresh();
                    self.load(arguments[0], B);
                    self.trap_unless_nonzero(B, TRAP_NULL_REFERENCE);
                    self.load_signed(arguments[1], C, bits);
                    self.memory(true, "", D, B, i64::from(ARRAY_LENGTH_OFFSET), 8);
                    self.line(&format!("cmp {C}, {D}"));
                    self.line(&format!("b.lo {fine}"));
                    self.trap(TRAP_INDEX_OUT_OF_BOUNDS);
                    self.label(&fine);
                    self.constant(size, D);
                    self.line(&format!("madd {B}, {C}, {D}, {B}"));
                    let offset = i64::from(ARRAY_HEADER);
                    if matches!(instruction.operation, Operation::StoreElement(_)) {
                        self.load(arguments[2], A);
                        self.memory_store(*ty, A, B, offset);
                    } else {
                        self.memory_load(*ty, B, offset, A);
                        self.store_result(A, result);
                    }
                }
                Operation::ArrayLength => {
                    let bits = shared::type_bits(result_type.unwrap_or(Type::I32));
                    self.load(arguments[0], B);
                    self.trap_unless_nonzero(B, TRAP_NULL_REFERENCE);
                    self.memory(true, "", A, B, i64::from(ARRAY_LENGTH_OFFSET), 8);
                    self.mask(A, bits);
                    self.store_result(A, result);
                }
                Operation::InstanceOf(class) => {
                    let done = self.fresh();
                    let scan = self.fresh();
                    let found = self.fresh();
                    self.load(arguments[0], B);
                    self.line(&format!("mov {A}, xzr"));
                    self.line(&format!("cbz {B}, {done}"));
                    self.constant(u64::from(class.0), F);
                    self.line(&format!("ldr {C}, [{B}]"));
                    self.line(&format!("ldr {E}, [{C}, #8]"));
                    self.line(&format!("ldr {B}, [{C}, #16]"));
                    self.label(&scan);
                    self.line(&format!("cbz {B}, {done}"));
                    self.line(&format!("ldr w12, [{E}]"));
                    self.line("cmp w12, w14");
                    self.line(&format!("b.eq {found}"));
                    self.line(&format!("add {E}, {E}, #4"));
                    self.line(&format!("sub {B}, {B}, #1"));
                    self.line(&format!("b {scan}"));
                    self.label(&found);
                    self.line("mov w9, #1");
                    self.label(&done);
                    self.store_result(A, result);
                }
                Operation::VirtualMethod(_, slot) => {
                    let fine = self.fresh();
                    let present = self.fresh();
                    self.load(arguments[0], B);
                    self.trap_unless_nonzero(B, TRAP_NULL_REFERENCE);
                    self.line(&format!("ldr {C}, [{B}]"));
                    self.line(&format!("ldr {D}, [{C}, #32]"));
                    self.constant(u64::from(*slot), E);
                    self.line(&format!("cmp {D}, {E}"));
                    self.line(&format!("b.hi {fine}"));
                    self.trap(TRAP_ABSTRACT_METHOD);
                    self.label(&fine);
                    self.line(&format!("ldr {C}, [{C}, #24]"));
                    self.memory(true, "", A, C, 8 * i64::from(*slot), 8);
                    self.line(&format!("cbnz {A}, {present}"));
                    self.trap(TRAP_ABSTRACT_METHOD);
                    self.label(&present);
                    self.store_result(A, result);
                }
                Operation::InterfaceMethod(class, slot) => {
                    let scan = self.fresh();
                    let found = self.fresh();
                    let fine = self.fresh();
                    let present = self.fresh();
                    let next = self.fresh();
                    self.load(arguments[0], B);
                    self.trap_unless_nonzero(B, TRAP_NULL_REFERENCE);
                    self.constant(u64::from(class.0), D);
                    self.line(&format!("ldr {C}, [{B}]"));
                    self.line(&format!("ldr {E}, [{C}, #40]"));
                    self.line(&format!("ldr {F}, [{C}, #48]"));
                    self.label(&scan);
                    self.line(&format!("cbnz {F}, {found}"));
                    self.trap(TRAP_MISSING_IMPLEMENTATION);
                    self.label(&found);
                    self.line(&format!("ldr x15, [{E}]"));
                    self.line(&format!("cmp x15, {D}"));
                    self.line(&format!("b.eq {next}"));
                    self.line(&format!("add {E}, {E}, #24"));
                    self.line(&format!("sub {F}, {F}, #1"));
                    self.line(&format!("b {scan}"));
                    self.label(&next);
                    self.line(&format!("ldr x15, [{E}, #16]"));
                    self.constant(u64::from(*slot), D);
                    self.line(&format!("cmp x15, {D}"));
                    self.line(&format!("b.hi {fine}"));
                    self.trap(TRAP_ABSTRACT_METHOD);
                    self.label(&fine);
                    self.line(&format!("ldr {C}, [{E}, #8]"));
                    self.memory(true, "", A, C, 8 * i64::from(*slot), 8);
                    self.line(&format!("cbnz {A}, {present}"));
                    self.trap(TRAP_ABSTRACT_METHOD);
                    self.label(&present);
                    self.store_result(A, result);
                }
                Operation::SameObject => {
                    self.load(arguments[0], A);
                    self.load(arguments[1], B);
                    self.line(&format!("cmp {A}, {B}"));
                    self.line("cset w9, eq");
                    self.store_result(A, result);
                }
                Operation::Trap(code) => {
                    self.constant(TRAP_EXPLICIT as u64, "x0");
                    self.constant(*code as u64, "x1");
                    self.line("bl __omni_trap");
                }
            }
            Ok(())
        }

        fn integer_binary(
            &mut self,
            op: BinaryOp,
            bits: u32,
            left: Value,
            right: Value,
            result: Option<Value>,
        ) {
            let wide = bits >= 64;
            let highest = if wide {
                i64::MAX
            } else {
                (1i64 << (bits.max(1) - 1)) - 1
            };
            let lowest = if wide { i64::MIN } else { -highest - 1 };
            match op {
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::And
                | BinaryOp::Or
                | BinaryOp::Xor => {
                    self.load(left, A);
                    self.load(right, B);
                    let mnemonic = match op {
                        BinaryOp::Add => "add",
                        BinaryOp::Subtract => "sub",
                        BinaryOp::Multiply => "mul",
                        BinaryOp::And => "and",
                        BinaryOp::Or => "orr",
                        _ => "eor",
                    };
                    self.line(&format!("{mnemonic} {A}, {A}, {B}"));
                    self.mask(A, bits);
                }
                BinaryOp::DivideSigned | BinaryOp::RemainderSigned => {
                    self.load_signed(left, A, bits);
                    self.load_signed(right, B, bits);
                    self.trap_unless_nonzero(B, TRAP_DIVISION_BY_ZERO);
                    if op == BinaryOp::DivideSigned {
                        self.line(&format!("sdiv {A}, {A}, {B}"));
                    } else {
                        self.line(&format!("sdiv {C}, {A}, {B}"));
                        self.line(&format!("msub {A}, {C}, {B}, {A}"));
                    }
                    self.mask(A, bits);
                }
                BinaryOp::DivideUnsigned | BinaryOp::RemainderUnsigned => {
                    self.load(left, A);
                    self.load(right, B);
                    self.trap_unless_nonzero(B, TRAP_DIVISION_BY_ZERO);
                    if op == BinaryOp::DivideUnsigned {
                        self.line(&format!("udiv {A}, {A}, {B}"));
                    } else {
                        self.line(&format!("udiv {C}, {A}, {B}"));
                        self.line(&format!("msub {A}, {C}, {B}, {A}"));
                    }
                }
                BinaryOp::ShiftLeft
                | BinaryOp::ShiftRightLogical
                | BinaryOp::ShiftRightArithmetic => {
                    if op == BinaryOp::ShiftRightArithmetic {
                        self.load_signed(left, A, bits);
                    } else {
                        self.load(left, A);
                    }
                    self.load(right, B);
                    let amount = bits.max(1) - 1;
                    if amount == 0 {
                        self.line(&format!("mov {B}, xzr"));
                    } else {
                        self.line(&format!("and {B}, {B}, #{amount}"));
                    }
                    let mnemonic = match op {
                        BinaryOp::ShiftLeft => "lsl",
                        BinaryOp::ShiftRightLogical => "lsr",
                        _ => "asr",
                    };
                    self.line(&format!("{mnemonic} {A}, {A}, {B}"));
                    self.mask(A, bits);
                }
                BinaryOp::RotateLeft | BinaryOp::RotateRight => {
                    self.load(left, A);
                    self.load(right, B);
                    let left_turn = op == BinaryOp::RotateLeft;
                    if bits >= 32 {
                        let (a, b) = if bits == 64 {
                            (A.to_string(), B.to_string())
                        } else {
                            (narrow(A), narrow(B))
                        };
                        if left_turn {
                            self.line(&format!("neg {b}, {b}"));
                        }
                        self.line(&format!("ror {a}, {a}, {b}"));
                    } else if bits >= 8 {
                        self.line(&format!("and {B}, {B}, #{}", bits - 1));
                        self.constant(u64::from(bits), D);
                        self.line(&format!("sub {D}, {D}, {B}"));
                        let (first, second) = if left_turn {
                            ("lsl", "lsr")
                        } else {
                            ("lsr", "lsl")
                        };
                        self.line(&format!("{first} {C}, {A}, {B}"));
                        self.line(&format!("{second} {A}, {A}, {D}"));
                        self.line(&format!("orr {A}, {A}, {C}"));
                        self.mask(A, bits);
                    }
                }
                BinaryOp::MinimumSigned | BinaryOp::MaximumSigned => {
                    self.load_signed(left, A, bits);
                    self.load_signed(right, B, bits);
                    self.line(&format!("cmp {A}, {B}"));
                    let condition = if op == BinaryOp::MinimumSigned {
                        "lt"
                    } else {
                        "gt"
                    };
                    self.line(&format!("csel {A}, {A}, {B}, {condition}"));
                    self.mask(A, bits);
                }
                BinaryOp::MinimumUnsigned | BinaryOp::MaximumUnsigned => {
                    self.load(left, A);
                    self.load(right, B);
                    self.line(&format!("cmp {A}, {B}"));
                    let condition = if op == BinaryOp::MinimumUnsigned {
                        "lo"
                    } else {
                        "hi"
                    };
                    self.line(&format!("csel {A}, {A}, {B}, {condition}"));
                }
                BinaryOp::AddSaturatingSigned | BinaryOp::SubtractSaturatingSigned => {
                    let adding = op == BinaryOp::AddSaturatingSigned;
                    let mnemonic = if adding { "adds" } else { "subs" };
                    self.load_signed(left, A, bits);
                    self.load_signed(right, B, bits);
                    if wide {
                        let done = self.fresh();
                        self.line(&format!("{mnemonic} {A}, {A}, {B}"));
                        self.line(&format!("b.vc {done}"));
                        self.constant(i64::MAX as u64, A);
                        self.line(&format!("cmp {B}, #0"));
                        self.line(&format!("b.{} {done}", if adding { "pl" } else { "mi" }));
                        self.constant(i64::MIN as u64, A);
                        self.label(&done);
                    } else {
                        self.line(&format!("{} {A}, {A}, {B}", &mnemonic[..3]));
                        self.constant(highest as u64, B);
                        self.line(&format!("cmp {A}, {B}"));
                        self.line(&format!("csel {A}, {B}, {A}, gt"));
                        self.constant(lowest as u64, B);
                        self.line(&format!("cmp {A}, {B}"));
                        self.line(&format!("csel {A}, {B}, {A}, lt"));
                        self.mask(A, bits);
                    }
                }
                BinaryOp::AddSaturatingUnsigned => {
                    self.load(left, A);
                    self.load(right, B);
                    if wide {
                        self.line(&format!("adds {A}, {A}, {B}"));
                        self.line(&format!("csinv {A}, {A}, xzr, lo"));
                    } else {
                        self.line(&format!("add {A}, {A}, {B}"));
                        self.constant(shared::all_ones(bits), B);
                        self.line(&format!("cmp {A}, {B}"));
                        self.line(&format!("csel {A}, {B}, {A}, hi"));
                    }
                }
                BinaryOp::SubtractSaturatingUnsigned => {
                    self.load(left, A);
                    self.load(right, B);
                    self.line(&format!("subs {A}, {A}, {B}"));
                    self.line(&format!("csel {A}, {A}, xzr, hs"));
                }
                BinaryOp::AddOverflowsSigned
                | BinaryOp::SubtractOverflowsSigned
                | BinaryOp::MultiplyOverflowsSigned => {
                    self.load_signed(left, A, bits);
                    self.load_signed(right, B, bits);
                    let multiply = op == BinaryOp::MultiplyOverflowsSigned;
                    if wide && multiply {
                        self.line(&format!("mul {C}, {A}, {B}"));
                        self.line(&format!("smulh {D}, {A}, {B}"));
                        self.line(&format!("cmp {D}, {C}, asr #63"));
                        self.line("cset w9, ne");
                    } else if wide {
                        let mnemonic = if op == BinaryOp::AddOverflowsSigned {
                            "adds"
                        } else {
                            "subs"
                        };
                        self.line(&format!("{mnemonic} {A}, {A}, {B}"));
                        self.line("cset w9, vs");
                    } else {
                        let mnemonic = match op {
                            BinaryOp::AddOverflowsSigned => "add",
                            BinaryOp::SubtractOverflowsSigned => "sub",
                            _ => "mul",
                        };
                        self.line(&format!("{mnemonic} {A}, {A}, {B}"));
                        self.line(&format!("mov {C}, {A}"));
                        self.sign_extend_register(C, bits);
                        self.line(&format!("cmp {A}, {C}"));
                        self.line("cset w9, ne");
                    }
                }
                BinaryOp::AddOverflowsUnsigned | BinaryOp::MultiplyOverflowsUnsigned => {
                    self.load(left, A);
                    self.load(right, B);
                    let adding = op == BinaryOp::AddOverflowsUnsigned;
                    if wide {
                        if adding {
                            self.line(&format!("adds {A}, {A}, {B}"));
                            self.line("cset w9, hs");
                        } else {
                            self.line(&format!("umulh {A}, {A}, {B}"));
                            self.line(&format!("cmp {A}, #0"));
                            self.line("cset w9, ne");
                        }
                    } else {
                        if adding {
                            self.line(&format!("add {A}, {A}, {B}"));
                        } else {
                            self.line(&format!("mul {A}, {A}, {B}"));
                        }
                        self.line(&format!("lsr {A}, {A}, #{bits}"));
                        self.line(&format!("cmp {A}, #0"));
                        self.line("cset w9, ne");
                    }
                }
                BinaryOp::SubtractOverflowsUnsigned => {
                    self.load(left, A);
                    self.load(right, B);
                    self.line(&format!("cmp {A}, {B}"));
                    self.line("cset w9, lo");
                }
                BinaryOp::MultiplyHighSigned | BinaryOp::MultiplyHighUnsigned => {
                    let signed = op == BinaryOp::MultiplyHighSigned;
                    if signed {
                        self.load_signed(left, A, bits);
                        self.load_signed(right, B, bits);
                    } else {
                        self.load(left, A);
                        self.load(right, B);
                    }
                    if wide {
                        let mnemonic = if signed { "smulh" } else { "umulh" };
                        self.line(&format!("{mnemonic} {A}, {A}, {B}"));
                    } else {
                        self.line(&format!("mul {A}, {A}, {B}"));
                        self.line(&format!(
                            "{} {A}, {A}, #{bits}",
                            if signed { "asr" } else { "lsr" }
                        ));
                        self.mask(A, bits);
                    }
                }
                _ => {}
            }
            self.store_result(A, result);
        }

        fn integer_unary(&mut self, op: UnaryOp, bits: u32, value: Value, result: Option<Value>) {
            match op {
                UnaryOp::Negate => {
                    self.load(value, A);
                    self.line(&format!("neg {A}, {A}"));
                    self.mask(A, bits);
                }
                UnaryOp::Not => {
                    self.load(value, A);
                    self.line(&format!("mvn {A}, {A}"));
                    self.mask(A, bits);
                }
                UnaryOp::PopulationCount => {
                    self.load(value, A);
                    self.line(&format!("fmov d16, {A}"));
                    self.line("cnt v16.8b, v16.8b");
                    self.line("addv b16, v16.8b");
                    self.line("fmov w9, s16");
                }
                UnaryOp::LeadingZeros => {
                    self.load(value, A);
                    self.line(&format!("clz {A}, {A}"));
                    if bits < 64 {
                        self.line(&format!("sub {A}, {A}, #{}", 64 - bits));
                    }
                }
                UnaryOp::TrailingZeros => {
                    self.load(value, A);
                    self.line(&format!("rbit {A}, {A}"));
                    self.line(&format!("clz {A}, {A}"));
                    if bits < 64 {
                        self.constant(u64::from(bits), B);
                        self.line(&format!("cmp {A}, {B}"));
                        self.line(&format!("csel {A}, {A}, {B}, lo"));
                    }
                }
                UnaryOp::ByteSwap => {
                    self.load(value, A);
                    match bits {
                        16 => self.line("rev16 w9, w9"),
                        32 => self.line("rev w9, w9"),
                        64 => self.line("rev x9, x9"),
                        _ => {}
                    }
                }
                UnaryOp::BitReverse => {
                    self.load(value, A);
                    self.line(&format!("rbit {A}, {A}"));
                    if bits < 64 {
                        self.line(&format!("lsr {A}, {A}, #{}", 64 - bits));
                    }
                }
                UnaryOp::AbsoluteSigned => {
                    self.load_signed(value, A, bits);
                    self.line(&format!("cmp {A}, #0"));
                    self.line(&format!("cneg {A}, {A}, lt"));
                    self.mask(A, bits);
                }
                _ => self.load(value, A),
            }
            self.store_result(A, result);
        }

        fn float_binary(
            &mut self,
            op: BinaryOp,
            ty: Type,
            left: Value,
            right: Value,
            result: Option<Value>,
        ) {
            let Some(result) = result else {
                return;
            };
            let first = float_register(ty, 16);
            let second = float_register(ty, 17);
            match op {
                BinaryOp::FloatAdd
                | BinaryOp::FloatSubtract
                | BinaryOp::FloatMultiply
                | BinaryOp::FloatDivide => {
                    self.load_float(left, ty, 16);
                    self.load_float(right, ty, 17);
                    let mnemonic = match op {
                        BinaryOp::FloatAdd => "fadd",
                        BinaryOp::FloatSubtract => "fsub",
                        BinaryOp::FloatMultiply => "fmul",
                        _ => "fdiv",
                    };
                    self.line(&format!("{mnemonic} {first}, {first}, {second}"));
                    self.canonical_nan(16, ty);
                    self.store_float(16, result, ty);
                }
                BinaryOp::FloatRemainder => self.float_remainder(ty, left, right, result),
                BinaryOp::FloatMinimum
                | BinaryOp::FloatMaximum
                | BinaryOp::FloatMinimumNumber
                | BinaryOp::FloatMaximumNumber => {
                    let lower = matches!(op, BinaryOp::FloatMinimum | BinaryOp::FloatMinimumNumber);
                    let number = matches!(
                        op,
                        BinaryOp::FloatMinimumNumber | BinaryOp::FloatMaximumNumber
                    );
                    let mnemonic = if lower { "fmin" } else { "fmax" };
                    self.load_float(left, ty, 16);
                    self.load_float(right, ty, 17);
                    if number {
                        let take_right = self.fresh();
                        let done = self.fresh();
                        self.line(&format!("fcmp {first}, {first}"));
                        self.line(&format!("b.vs {take_right}"));
                        self.line(&format!("fcmp {second}, {second}"));
                        self.line(&format!("b.vs {done}"));
                        self.line(&format!("{mnemonic} {first}, {first}, {second}"));
                        self.line(&format!("b {done}"));
                        self.label(&take_right);
                        self.line(&format!("fmov {first}, {second}"));
                        self.label(&done);
                    } else {
                        self.line(&format!("{mnemonic} {first}, {first}, {second}"));
                    }
                    self.canonical_nan(16, ty);
                    self.store_float(16, result, ty);
                }
                BinaryOp::CopySign => {
                    self.load(left, A);
                    self.load(right, B);
                    if shared::single(ty) {
                        self.line("and w9, w9, #0x7fffffff");
                        self.line("and w10, w10, #0x80000000");
                        self.line("orr w9, w9, w10");
                    } else {
                        self.line("and x9, x9, #0x7fffffffffffffff");
                        self.line("and x10, x10, #0x8000000000000000");
                        self.line("orr x9, x9, x10");
                    }
                    self.store(A, result);
                }
                _ => {}
            }
        }

        fn float_remainder(&mut self, ty: Type, left: Value, right: Value, result: Value) {
            let nan = self.fresh();
            let keep = self.fresh();
            let fin = self.fresh();
            let grow = self.fresh();
            let shrink = self.fresh();
            let skip = self.fresh();
            self.load_float(left, ty, 16);
            self.load_float(right, ty, 17);
            if shared::single(ty) {
                self.line("fcvt d16, s16");
                self.line("fcvt d17, s17");
            }
            self.line("fabs d18, d16");
            self.line("fabs d19, d17");
            self.line("fcmp d18, d18");
            self.line(&format!("b.vs {nan}"));
            self.line("fcmp d19, d19");
            self.line(&format!("b.vs {nan}"));
            self.line("fcmp d19, #0.0");
            self.line(&format!("b.eq {nan}"));
            self.line("movz x16, #0x7ff0, lsl #48");
            self.line("fmov d20, x16");
            self.line("fcmp d18, d20");
            self.line(&format!("b.eq {nan}"));
            self.line("fcmp d18, d19");
            self.line(&format!("b.lo {keep}"));
            self.line("fmov d21, d19");
            self.label(&grow);
            self.line("fadd d22, d21, d21");
            self.line("fcmp d22, d18");
            self.line(&format!("b.hi {shrink}"));
            self.line("fmov d21, d22");
            self.line(&format!("b {grow}"));
            self.label(&shrink);
            self.line("fcmp d18, d21");
            self.line(&format!("b.lo {skip}"));
            self.line("fsub d18, d18, d21");
            self.label(&skip);
            self.line("fcmp d21, d19");
            self.line(&format!("b.eq {fin}"));
            self.line("fmov d22, #0.5");
            self.line("fmul d21, d21, d22");
            self.line(&format!("b {shrink}"));
            self.label(&fin);
            self.line("fmov x9, d18");
            self.line("fmov x10, d16");
            self.line("and x10, x10, #0x8000000000000000");
            self.line("orr x9, x9, x10");
            self.line("fmov d16, x9");
            self.line(&format!("b {keep}"));
            self.label(&nan);
            self.nan_into(Type::F64, 16);
            self.label(&keep);
            if shared::single(ty) {
                self.line("fcvt s16, d16");
            }
            self.store_float(16, result, ty);
        }

        fn float_unary(&mut self, op: UnaryOp, ty: Type, value: Value, result: Option<Value>) {
            let Some(result) = result else {
                return;
            };
            let register = float_register(ty, 16);
            match op {
                UnaryOp::FloatNegate | UnaryOp::FloatAbsolute => {
                    self.load(value, A);
                    let negate = op == UnaryOp::FloatNegate;
                    let line = match (shared::single(ty), negate) {
                        (true, true) => "eor w9, w9, #0x80000000",
                        (true, false) => "and w9, w9, #0x7fffffff",
                        (false, true) => "eor x9, x9, #0x8000000000000000",
                        (false, false) => "and x9, x9, #0x7fffffffffffffff",
                    };
                    self.line(line);
                    self.store(A, result);
                }
                UnaryOp::SquareRoot
                | UnaryOp::Floor
                | UnaryOp::Ceiling
                | UnaryOp::TruncateFloat
                | UnaryOp::RoundNearestEven => {
                    let mnemonic = match op {
                        UnaryOp::SquareRoot => "fsqrt",
                        UnaryOp::Floor => "frintm",
                        UnaryOp::Ceiling => "frintp",
                        UnaryOp::TruncateFloat => "frintz",
                        _ => "frintn",
                    };
                    self.load_float(value, ty, 16);
                    self.line(&format!("{mnemonic} {register}, {register}"));
                    self.canonical_nan(16, ty);
                    self.store_float(16, result, ty);
                }
                _ => {
                    self.load(value, A);
                    self.store(A, result);
                }
            }
        }

        fn float_compare(
            &mut self,
            comparison: FloatComparison,
            ty: Type,
            arguments: &[Value],
            result: Option<Value>,
        ) {
            self.load_float(arguments[0], ty, 16);
            self.load_float(arguments[1], ty, 17);
            let first = float_register(ty, 16);
            let second = float_register(ty, 17);
            self.line(&format!("fcmp {first}, {second}"));
            let pair = |this: &mut Self, one: &str, two: &str| {
                this.line(&format!("cset w9, {one}"));
                this.line(&format!("cset w10, {two}"));
                this.line("orr w9, w9, w10");
            };
            match comparison {
                FloatComparison::OrderedNotEqual => pair(self, "mi", "gt"),
                FloatComparison::UnorderedEqual => pair(self, "eq", "vs"),
                other => {
                    let condition = match other {
                        FloatComparison::OrderedEqual => "eq",
                        FloatComparison::UnorderedNotEqual => "ne",
                        FloatComparison::OrderedLess => "mi",
                        FloatComparison::OrderedLessEqual => "ls",
                        FloatComparison::OrderedGreater => "gt",
                        FloatComparison::OrderedGreaterEqual => "ge",
                        FloatComparison::Ordered => "vc",
                        FloatComparison::Unordered => "vs",
                        FloatComparison::UnorderedLess => "lt",
                        FloatComparison::UnorderedLessEqual => "le",
                        FloatComparison::UnorderedGreater => "hi",
                        _ => "pl",
                    };
                    self.line(&format!("cset w9, {condition}"));
                }
            }
            self.store_result(A, result);
        }

        fn convert(
            &mut self,
            conversion: Conversion,
            value: Value,
            from: Type,
            to: Type,
            result: Option<Value>,
        ) {
            let Some(result) = result else {
                return;
            };
            let from_bits = shared::type_bits(from);
            let to_bits = shared::type_bits(to);
            match conversion {
                Conversion::ZeroExtend | Conversion::IntegerToPointer => {
                    self.load(value, A);
                    self.store(A, result);
                }
                Conversion::SignExtend => {
                    self.load_signed(value, A, from_bits);
                    self.mask(A, to_bits);
                    self.store(A, result);
                }
                Conversion::Truncate | Conversion::PointerToInteger => {
                    self.load(value, A);
                    self.mask(A, to_bits);
                    self.store(A, result);
                }
                Conversion::Bitcast => {
                    let size = self.layout.size_of(from).min(self.layout.size_of(to));
                    self.load(value, A);
                    self.mask(
                        A,
                        match size {
                            1 => 8,
                            2 => 16,
                            4 => 32,
                            _ => 64,
                        },
                    );
                    self.store(A, result);
                }
                Conversion::FloatExtend | Conversion::FloatTruncate => {
                    if from == to {
                        self.load(value, A);
                        self.store(A, result);
                        return;
                    }
                    self.load_float(value, from, 16);
                    let source = float_register(from, 16);
                    let target = float_register(to, 16);
                    self.line(&format!("fcvt {target}, {source}"));
                    self.canonical_nan(16, to);
                    self.store_float(16, result, to);
                }
                Conversion::SignedToFloat | Conversion::UnsignedToFloat => {
                    let signed = conversion == Conversion::SignedToFloat;
                    if signed {
                        self.load_signed(value, A, from_bits);
                    } else {
                        self.load(value, A);
                    }
                    let target = float_register(to, 16);
                    let mnemonic = if signed { "scvtf" } else { "ucvtf" };
                    self.line(&format!("{mnemonic} {target}, {A}"));
                    self.store_float(16, result, to);
                }
                Conversion::FloatToSigned
                | Conversion::FloatToUnsigned
                | Conversion::FloatToSignedSaturating
                | Conversion::FloatToUnsignedSaturating => {
                    let signed = matches!(
                        conversion,
                        Conversion::FloatToSigned | Conversion::FloatToSignedSaturating
                    );
                    let saturating = matches!(
                        conversion,
                        Conversion::FloatToSignedSaturating | Conversion::FloatToUnsignedSaturating
                    );
                    let bits = to_bits;
                    let mnemonic = if signed { "fcvtzs" } else { "fcvtzu" };
                    self.load_float(value, from, 16);
                    if shared::single(from) {
                        self.line("fcvt d16, s16");
                    }
                    if saturating {
                        self.line(&format!("{mnemonic} {A}, d16"));
                        if bits < 64 {
                            if signed {
                                self.constant(shared::all_ones(bits - 1), B);
                                self.line(&format!("cmp {A}, {B}"));
                                self.line(&format!("csel {A}, {B}, {A}, gt"));
                                self.constant((-(1i64 << (bits - 1))) as u64, B);
                                self.line(&format!("cmp {A}, {B}"));
                                self.line(&format!("csel {A}, {B}, {A}, lt"));
                            } else {
                                self.constant(shared::all_ones(bits), B);
                                self.line(&format!("cmp {A}, {B}"));
                                self.line(&format!("csel {A}, {B}, {A}, hi"));
                            }
                        }
                        self.mask(A, bits);
                    } else {
                        let (lowest, limit) = if signed {
                            (-(2f64.powi(bits as i32 - 1)), 2f64.powi(bits as i32 - 1))
                        } else {
                            (0.0, 2f64.powi(bits as i32))
                        };
                        let bad = self.fresh();
                        let done = self.fresh();
                        self.line("fcmp d16, d16");
                        self.line(&format!("b.vs {bad}"));
                        self.line("frintz d17, d16");
                        self.constant(lowest.to_bits(), "x16");
                        self.line("fmov d18, x16");
                        self.line("fcmp d17, d18");
                        self.line(&format!("b.mi {bad}"));
                        self.constant(limit.to_bits(), "x16");
                        self.line("fmov d18, x16");
                        self.line("fcmp d17, d18");
                        self.line(&format!("b.ge {bad}"));
                        self.line(&format!("{mnemonic} {A}, d17"));
                        self.mask(A, bits);
                        self.line(&format!("b {done}"));
                        self.label(&bad);
                        self.trap(TRAP_CONVERSION_RANGE);
                        self.label(&done);
                    }
                    self.store(A, result);
                }
            }
        }
    }

    pub fn emit_assembly(module: &Module) -> Result<String, Diagnostic> {
        let mut emitter = Arm {
            base: Emitter {
                module,
                layout: module.layout.data(),
                out: String::new(),
                labels: 0,
                fields: BTreeMap::new(),
            },
        };
        emitter.emit_module()?;
        Ok(emitter.base.out)
    }

    pub fn start_assembly(entry: &str) -> String {
        format!(
            "\t.text\n\t.globl _start\n\t.type _start, @function\n_start:\n\tmov x29, xzr\n\tmov x30, xzr\n\tmov x9, sp\n\tand x9, x9, #0xfffffffffffffff0\n\tmov sp, x9\n\tbl {entry}\n\tadrp x16, __omni_pending\n\tldr x16, [x16, :lo12:__omni_pending]\n\tcbz x16, .Lstart_exit\n\tbl __omni_uncaught\n.Lstart_exit:\n\tmov x8, #94\n\tsvc #0\n\tbrk #0\n"
        )
    }

    pub fn assemble(text: &str) -> Result<Vec<u8>, Diagnostic> {
        shared::assemble_for(TRIPLE, text)
    }

    pub fn runtime_text() -> String {
        let mut text = shared::RUNTIME.to_string();
        for (from, to) in [(231, 94), (228, 113), (9, 222), (1, 64)] {
            let old = format!("%number: i64 = const {from}\n");
            let new = format!("%number: i64 = const {to}\n");
            text = text.replace(&old, &new);
        }
        text
    }

    pub fn runtime_module() -> Result<Module, Diagnostic> {
        Module::from_text(&runtime_text())
    }

    pub fn build_executable(module: &Module, entry: &str) -> Result<Vec<u8>, Diagnostic> {
        let provided = shared::runtime_names();
        let defined: BTreeSet<&str> = module
            .functions
            .iter()
            .filter(|function| !function.is_declaration())
            .map(|function| function.name.as_str())
            .collect();
        let mut stubs = String::new();
        for (index, function) in module.functions.iter().enumerate() {
            if function.is_declaration()
                && !provided.contains(&function.name)
                && !defined.contains(function.name.as_str())
            {
                let symbol = shared::function_symbol(module, FunctionId(index as u32));
                stubs.push_str(&format!(
                    "\t.text\n\t.globl {symbol}\n{symbol}:\n\tadrp x0, .Lmissing{index}\n\tadd x0, x0, :lo12:.Lmissing{index}\n\tmov x1, #{}\n\tbl __omni_missing\n\t.section .rodata\n.Lmissing{index}:\n",
                    function.name.len()
                ));
                for chunk in function.name.as_bytes().chunks(16) {
                    let listed: Vec<String> = chunk.iter().map(|byte| byte.to_string()).collect();
                    stubs.push_str(&format!("\t.byte {}\n", listed.join(", ")));
                }
            }
        }
        let runtime = runtime_module()?;
        let user = assemble(&emit_assembly(module)?)?;
        let support = assemble(&emit_assembly(&runtime)?)?;
        let start = assemble(&format!("{}{stubs}", start_assembly(entry)))?;
        shared::link_program(user, support, start)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AttributeSubsection {
    pub name: String,
    pub optional: u8,
    pub kind: u8,
    pub active: bool,
    pub content: Vec<(u32, Option<u32>, String)>,
}

#[derive(Clone, Debug, Default)]
pub struct AttributeSet {
    pub subsections: Vec<AttributeSubsection>,
}

impl AttributeSet {
    pub fn subsection(&mut self, name: &str, optional: u8, kind: u8) {
        if !self.subsections.iter().any(|held| held.name == name) {
            self.subsections.push(AttributeSubsection {
                name: name.to_string(),
                optional,
                kind,
                active: false,
                content: Vec::new(),
            });
        }
        for held in &mut self.subsections {
            held.active = held.name == name;
        }
    }

    pub fn active(&self) -> Option<&AttributeSubsection> {
        self.subsections.iter().find(|held| held.active)
    }

    pub fn by_name(&self, name: &str) -> Option<&AttributeSubsection> {
        self.subsections.iter().find(|held| held.name == name)
    }

    pub fn attribute(&mut self, vendor: &str, tag: u32, value: Option<u32>, text: &str) {
        let Some(held) = self
            .subsections
            .iter_mut()
            .find(|held| held.name == vendor && held.active)
        else {
            return;
        };
        if let Some(item) = held.content.iter_mut().find(|item| item.0 == tag) {
            item.1 = value;
            item.2 = if value.is_some() {
                String::new()
            } else {
                text.to_string()
            };
            return;
        }
        held.content.push((tag, value, text.to_string()));
    }

    pub fn encode(&self) -> Vec<u8> {
        if self.subsections.is_empty() {
            return Vec::new();
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
        let mut out = vec![b'A'];
        for held in &self.subsections {
            let mut payload = Vec::new();
            payload.extend_from_slice(held.name.as_bytes());
            payload.push(0);
            payload.push(held.optional);
            payload.push(held.kind);
            for (tag, value, text) in &held.content {
                uleb(&mut payload, u64::from(*tag));
                match value {
                    Some(value) => uleb(&mut payload, u64::from(*value)),
                    None => {
                        payload.extend_from_slice(text.as_bytes());
                        payload.push(0);
                    }
                }
            }
            out.extend_from_slice(&((payload.len() + 4) as u32).to_le_bytes());
            out.extend_from_slice(&payload);
        }
        out
    }
}

impl<'s> Assembler<'s> {
    fn arm_directive_subsection(&mut self) -> bool {
        if !self.is(TokenKind::Identifier) {
            return self.token_error("subsection name not found");
        }
        let name = self.lexer.token.text.clone();
        self.lex();
        let existing = self.arm_attributes.by_name(&name).cloned();
        if self.is(TokenKind::EndOfStatement) {
            return match existing {
                Some(held) => {
                    self.arm_attributes.subsection(&name, held.optional, held.kind);
                    self.streamer
                        .emit_arm_subsection(&name, held.optional, held.kind);
                    false
                }
                None => self.token_error(&format!(
                    "Could not switch to subsection '{name}' using subsection name, subsection has not been defined"
                )),
            };
        }
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let optional_text = |value: u8| if value == 1 { "optional" } else { "required" };
        let optional;
        if self.is(TokenKind::Identifier) {
            optional =
                match self.lexer.token.text.as_str() {
                    "required" => 0u8,
                    "optional" => 1u8,
                    _ => return self.token_error(
                        "unknown AArch64 build attributes optionality, expected required|optional",
                    ),
                };
            if let Some(held) = &existing {
                if held.optional != optional {
                    return self.token_error(&format!(
                        "optionality mismatch! subsection '{name}' already exists with optionality defined as '{}' and not '{}'",
                        optional_text(held.optional),
                        optional_text(optional)
                    ));
                }
            }
        } else {
            return self.token_error("optionality parameter not found, expected required|optional");
        }
        if name == "aeabi_feature_and_bits" && optional == 0 {
            return self.token_error("aeabi_feature_and_bits must be marked as optional");
        }
        if name == "aeabi_pauthabi" && optional == 1 {
            return self.token_error("aeabi_pauthabi must be marked as required");
        }
        self.lex();
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let kind_text = |value: u8| if value == 0 { "uleb128" } else { "ntbs" };
        let kind;
        if self.is(TokenKind::Identifier) {
            kind = match self.lexer.token.text.as_str() {
                "uleb128" => 0u8,
                "ntbs" => 1u8,
                _ => {
                    return self.token_error(
                        "unknown AArch64 build attributes type, expected uleb128|ntbs",
                    )
                }
            };
            if let Some(held) = &existing {
                if held.kind != kind {
                    return self.token_error(&format!(
                        "type mismatch! subsection '{name}' already exists with type defined as '{}' and not '{}'",
                        kind_text(held.kind),
                        kind_text(kind)
                    ));
                }
            }
        } else {
            return self.token_error("type parameter not found, expected uleb128|ntbs");
        }
        if matches!(name.as_str(), "aeabi_feature_and_bits" | "aeabi_pauthabi") && kind == 1 {
            return self.token_error(&format!("{name} must be marked as ULEB128"));
        }
        self.lex();
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error(
                "unexpected token for AArch64 build attributes subsection header directive",
            );
        }
        self.arm_attributes.subsection(&name, optional, kind);
        self.streamer.emit_arm_subsection(&name, optional, kind);
        false
    }

    fn arm_directive_attribute(&mut self) -> bool {
        let Some(active) = self.arm_attributes.active().cloned() else {
            return self.token_error("no active subsection, build attribute can not be added");
        };
        let vendor = active.name.clone();
        let tag_name;
        let tag: u32;
        if self.is(TokenKind::Integer) {
            tag_name = String::new();
            tag = self.lexer.token.integer() as u32;
        } else if self.is(TokenKind::Identifier) {
            tag_name = self.lexer.token.text.clone();
            let found = match (vendor.as_str(), tag_name.as_str()) {
                ("aeabi_pauthabi", "Tag_PAuth_Platform") => Some(1),
                ("aeabi_pauthabi", "Tag_PAuth_Schema") => Some(2),
                ("aeabi_feature_and_bits", "Tag_Feature_BTI") => Some(0),
                ("aeabi_feature_and_bits", "Tag_Feature_PAC") => Some(1),
                ("aeabi_feature_and_bits", "Tag_Feature_GCS") => Some(2),
                _ => None,
            };
            match found {
                Some(value) => tag = value,
                None if matches!(vendor.as_str(), "aeabi_pauthabi" | "aeabi_feature_and_bits") => {
                    return self.token_error(&format!(
                        "unknown AArch64 build attribute '{tag_name}' for subsection '{vendor}'"
                    ));
                }
                None => {
                    return self.token_error(&format!(
                        "unrecognized Tag: '{tag_name}' \nExcept for public subsections, tags have to be an unsigned int."
                    ));
                }
            }
        } else {
            return self.token_error("AArch64 build attributes tag not found");
        }
        self.lex();
        if self.parse_token(TokenKind::Comma, "expected comma") {
            return true;
        }
        let mut number: Option<u32> = None;
        let mut text = String::new();
        if self.is(TokenKind::Integer) {
            if active.kind == 1 {
                return self.token_error(
                    "active subsection type is NTBS (string), found ULEB128 (unsigned)",
                );
            }
            number = Some(self.lexer.token.integer() as u32);
        } else if self.is(TokenKind::Identifier) || self.is(TokenKind::String) {
            if active.kind == 0 {
                return self.token_error(
                    "active subsection type is ULEB128 (unsigned), found NTBS (string)",
                );
            }
            text = self.lexer.token.text.clone();
        } else {
            return self.token_error("AArch64 build attributes value not found");
        }
        if vendor == "aeabi_feature_and_bits" && number.is_some_and(|value| value > 1) {
            return self.token_error(&format!(
                "unknown AArch64 build attributes Value for Tag '{tag_name}' options are 0|1"
            ));
        }
        self.lex();
        if !self.is(TokenKind::EndOfStatement) {
            return self.token_error(
                "unexpected token for AArch64 build attributes tag and value attribute directive",
            );
        }
        self.arm_attributes.attribute(&vendor, tag, number, &text);
        self.streamer
            .emit_arm_attribute(&vendor, tag, number, &text);
        false
    }
}

#[derive(Clone, Debug)]
pub struct PoolEntry {
    pub label: usize,
    pub value: Expression,
    pub size: u32,
    pub location: SourceLocation,
}

impl<'s> Assembler<'s> {
    fn arm_parse_literal_load(&mut self, operands: &mut Vec<ParsedOperand>) -> bool {
        let location = self.location();
        if self.arm_mnemonic != "ldr" {
            return self.token_error("unexpected token in operand");
        }
        self.lex();
        let start = self.lexer.token.start;
        let Ok((value, _)) = self.parse_expression() else {
            return true;
        };
        let register = match operands.get(1).and_then(arm_of) {
            Some(ArmOperand::Register {
                register,
                kind: RegKind::Scalar,
                ..
            }) => Some(*register),
            _ => None,
        };
        let Some(register) = register else {
            return self.error(location, "Only valid when first operand is register");
        };
        let wide = self.target.in_register_class("GPR64", register);
        if let Expression::Constant(constant) = &value {
            let mut immediate = *constant as u64;
            let mut shift = 0u32;
            let largest = if wide { 48 } else { 16 };
            while immediate > 0xffff && immediate.trailing_zeros() >= 16 {
                shift += 16;
                immediate >>= 16;
            }
            if shift <= largest && immediate <= 0xffff {
                operands[0].kind = ParsedKind::Token("movz".to_string());
                self.arm_push(
                    operands,
                    start,
                    ArmOperand::Immediate(Expression::Constant(immediate as i64)),
                );
                if shift != 0 {
                    self.arm_push(
                        operands,
                        start,
                        ArmOperand::ShiftExtend(ShiftExtendOperand {
                            kind: ShiftExtendKind::Lsl,
                            amount: shift,
                            explicit: true,
                        }),
                    );
                }
                return false;
            }
            let shifted = (immediate << shift) as i64;
            if !wide && !(i64::from(shifted as i32) == shifted || (shifted as u64) <= 0xffff_ffff) {
                return self.error(location, "Immediate too large for register");
            }
        }
        let size = if wide { 8 } else { 4 };
        let label = self.arm_pool_add(value, size, location);
        let name = self.context.symbols[label].name.clone();
        self.arm_push(
            operands,
            start,
            ArmOperand::Immediate(Expression::symbol(&name)),
        );
        false
    }

    fn arm_pool_add(&mut self, value: Expression, size: u32, location: SourceLocation) -> usize {
        let section = self
            .context
            .current_section()
            .map(|(section, _)| section)
            .unwrap_or(0);
        let slot = match self.arm_pools.iter().position(|(held, _)| *held == section) {
            Some(slot) => slot,
            None => {
                self.arm_pools.push((section, Vec::new()));
                self.arm_pools.len() - 1
            }
        };
        let cacheable = match &value {
            Expression::Constant(_) => true,
            Expression::Symbol { specifier, .. } => specifier.is_none(),
            _ => false,
        };
        if cacheable {
            if let Some(entry) = self.arm_pools[slot]
                .1
                .iter()
                .find(|entry| entry.value == value && entry.size == size)
            {
                return entry.label;
            }
        }
        let label = self.context.temporary_symbol();
        self.arm_pools[slot].1.push(PoolEntry {
            label,
            value,
            size,
            location,
        });
        label
    }

    fn arm_pool_emit(&mut self, section: usize) {
        let Some(slot) = self.arm_pools.iter().position(|(held, _)| *held == section) else {
            return;
        };
        let entries = std::mem::take(&mut self.arm_pools[slot].1);
        for entry in entries {
            self.streamer.emit_value_to_alignment(
                &mut self.context,
                u64::from(entry.size),
                0,
                1,
                0,
            );
            self.streamer
                .emit_label(&mut self.context, entry.label, Some(entry.location));
            self.streamer
                .emit_value(&mut self.context, &entry.value, entry.size, entry.location);
        }
    }

    pub(super) fn arm_pool_finish(&mut self) {
        let sections: Vec<usize> = self
            .arm_pools
            .iter()
            .filter(|(_, entries)| !entries.is_empty())
            .map(|(section, _)| *section)
            .collect();
        for section in sections {
            self.streamer.switch_section(&mut self.context, section, 0);
            self.arm_pool_emit(section);
        }
    }
}
