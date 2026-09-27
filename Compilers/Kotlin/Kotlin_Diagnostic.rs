pub const CATALOGUE: &[(&str, &str)] = &[
    ("EK001", "A comment was opened and never closed."),
    ("EK002", "A name in backticks was opened and never closed."),
    (
        "EK003",
        "A character that is not part of Kotlin appeared in the source.",
    ),
    ("EK004", "That is not a number Kotlin reads."),
    ("EK005", "That number is too large for a Long."),
    ("EK006", "That backslash escape is not one Kotlin knows."),
    (
        "EK007",
        "A character literal holds one character between single quotes.",
    ),
    (
        "EK008",
        "A string template was opened with ${ and never closed.",
    ),
    ("EK009", "A string was opened and never closed."),
    ("EK100", "A mark the language expects here is not there."),
    ("EK101", "A name was expected here."),
    (
        "EK102",
        "A declaration was expected here, at the top of a file.",
    ),
    ("EK103", "An enum entry was expected here."),
    ("EK104", "A class body was opened and never closed."),
    (
        "EK105",
        "A member declaration was expected here, inside a class body.",
    ),
    ("EK106", "A block was opened and never closed."),
    (
        "EK107",
        "Two statements on one line need a semicolon between them.",
    ),
    (
        "EK108",
        "A statement is not shaped the way Kotlin writes one.",
    ),
    (
        "EK109",
        "An expression is not shaped the way Kotlin writes one.",
    ),
    ("EK110", "A string template holds more than one expression."),
    ("EK111", "An expression was expected here."),
    ("EK112", "A when was opened and never closed."),
    (
        "EK113",
        "A call was expected after a function given in parentheses.",
    ),
    ("EK200", "A name is not a type this compilation knows."),
    ("EK201", "A package was used where a value belongs."),
    ("EK202", "super can only be used to reach a member."),
    ("EK203", "A when branch uses in without a subject."),
    (
        "EK204",
        "A break or a continue is not inside a loop it can leave.",
    ),
    (
        "EK205",
        "A return is not inside anything it can return from.",
    ),
    ("EK206", "There is no this here."),
    (
        "EK207",
        "A class name was used as a value, and it has no companion object.",
    ),
    ("EK210", "A name is not anything this code can see."),
    (
        "EK211",
        "A lambda given to an inline function can only be called.",
    ),
    ("EK212", "A property cannot be read here."),
    (
        "EK213",
        "That library property is inline only and has no body this compiler carries.",
    ),
    ("EK214", "A type depends on itself; write it out."),
    ("EK215", "This class cannot be constructed here."),
    ("EK216", "No function of that name takes these arguments."),
    (
        "EK217",
        "A reference to a member needs the value it is taken from.",
    ),
    ("EK218", "That is nothing which can be assigned to."),
    ("EK219", "That property cannot be assigned."),
    ("EK220", "A for loop cannot walk over this range."),
    ("EK221", "A destructuring declaration needs a value."),
    (
        "EK300",
        "Kotlin compiles through the build, not through this plugin surface.",
    ),
    (
        "EK301",
        "Kotlin source was found and the Kotlin standard library is not among the dependencies.",
    ),
    ("EK302", "A Kotlin source file is not text."),
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
