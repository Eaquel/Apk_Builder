use crate::compilers::x86_64::{target, Encoded, Encoder, MachineInstruction, Mode};
use crate::diag::Diagnostic;

pub const TABLE: &[u8] = include_bytes!("Tables/X86.bin");

pub fn encode(instruction: &MachineInstruction, sixteen_bit: bool) -> Result<Encoded, Diagnostic> {
    let mode = if sixteen_bit {
        Mode::Bits16
    } else {
        Mode::Bits32
    };
    Encoder::new(target()?, mode).encode(instruction)
}
