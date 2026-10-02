pub const CATALOGUE: &[(&str, &str)] = &[
    ("EI001", "The intermediate text could not be read."),
    ("EI002", "The intermediate binary could not be read."),
    ("EI003", "The intermediate form broke one of its rules."),
    ("EI004", "The program stopped at a trap."),
    ("EI005", "The program ran longer than it was allowed to."),
    ("EI006", "The program threw an exception nothing caught."),
    ("EI007", "The program called a function the host does not provide."),
    ("EI008", "The program used more stack than it was allowed to."),
    ("EI200", "The object file could not be read."),
    ("EI201", "The archive could not be read."),
    ("EI202", "The linker does not recognize this option."),
    ("EI203", "An option is missing its value."),
    ("EI204", "An option value is not valid."),
    ("EI205", "An input file could not be found."),
    ("EI206", "A symbol is defined more than once."),
    ("EI207", "A symbol is referenced but never defined."),
    ("EI208", "A relocation value does not fit in its field."),
    ("EI209", "The input targets a different machine than the output."),
    ("EI210", "The linker does not support this relocation."),
    ("EI212", "The linker does not support what this input asks for yet."),
    ("EI213", "No input files were given."),
    ("EI214", "A shared library cannot be linked statically."),
    ("EI215", "The input file is neither an object nor a shared library."),
    ("EI216", "A relocation section points at a section that does not exist."),
    ("EI217", "A section links to a section that does not exist."),
    ("EI218", "A symbol has a binding the linker does not know."),
    ("EI219", "A symbol points at a section that does not exist."),
    ("EI220", "Sections of different types are placed in one output section."),
    ("EI221", "Thread-local and ordinary sections are placed in one output section."),
    ("EI222", "An input redefines a symbol the linker defines."),
    ("EI223", "A symbol is defined in a section the linker discarded."),
    ("EI224", "A relocation cannot refer to an absolute symbol here."),
    ("EI225", "A protected symbol of a shared library cannot be preempted."),
    ("EI226", "A relocation against a shared library object needs a copy relocation, and copy relocations are turned off."),
    ("EI227", "A function of a shared library cannot be preempted here; it needs position independent code."),
    ("EI228", "A relocation cannot be used against this symbol; the code needs to be position independent."),
    ("EI229", "A thread-local relocation for executables cannot be used in a shared library."),
    ("EI230", "A relocation names a symbol that does not exist."),
    ("EI231", "A thread-local descriptor relocation is used with an instruction it does not fit."),
    ("EI232", "An initial executable thread-local relocation is used with an instruction it does not fit."),
    ("EI233", "A local dynamic thread-local relocation is not followed by the call it needs."),
    ("EI234", "A relocation value is not aligned the way its field needs."),
    ("EI235", "A relocation value cannot be encoded as an immediate."),
    ("EI236", "Inputs disagree on how floating point arguments are passed."),
    ("EI237", "Address assignment did not settle."),
    ("EI238", "This branch cannot reach its target on the architecture the inputs describe."),
    ("EI239", "The linker script could not be read."),
    ("EI240", "The exception frame section is malformed."),
    ("EI241", "A copy relocation cannot be made for this symbol."),
    ("EI242", "A version script names a symbol that is not defined."),
    ("EI243", "A symbol names a version that is not defined."),
    ("EI244", "A version script assigns a symbol to a second version."),
    ("EI245", "The wildcard pattern * is used more than once in the version script."),
    ("EI246", "The module definition file could not be read."),
    ("EI247", "The linker ignores an option it does not recognize."),
    ("EI248", "A relocation points outside its section or at a discarded section."),
    ("EI249", "The subsystem cannot be inferred and must be given."),
    ("EI251", "A common symbol is overridden."),
    ("EI252", "A common symbol is declared more than once."),
    ("EI253", "A branch and link relocation does not switch between ARM and Thumb state."),
    ("EI254", "The ARM attributes of an input could not be read."),
    ("EI255", "Two exports use the same ordinal."),
    ("EI256", "A symbol is exported more than once."),
    ("EI257", "Literal .edata sections override the exports."),
    ("EI258", "The target machine was not given, so x64 is assumed."),
    ("EI259", "A locally defined symbol is imported."),
    ("EI260", "An input is not compatible with safe exception handlers."),
];

pub fn english_of(code: &str) -> Option<&'static str> {
    CATALOGUE
        .binary_search_by(|(held, _)| (*held).cmp(code))
        .ok()
        .map(|at| CATALOGUE[at].1)
}

pub fn explain(code: &str) -> Option<String> {
    english_of(code).map(crate::speech::sentence)
}

pub fn known(code: &str) -> bool {
    english_of(code).is_some()
}
