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
