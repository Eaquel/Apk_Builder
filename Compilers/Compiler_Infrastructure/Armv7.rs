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
