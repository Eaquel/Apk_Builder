pub const CATALOGUE: &[(&str, &str)] = &[
    ("EI001", "The intermediate text could not be read."),
    ("EI002", "The intermediate binary could not be read."),
    ("EI003", "The intermediate form broke one of its rules."),
    ("EI004", "The program stopped at a trap."),
    ("EI005", "The program ran longer than it was allowed to."),
    ("EI006", "The program threw an exception nothing caught."),
    (
        "EI007",
        "The program called a function the host does not provide.",
    ),
    (
        "EI008",
        "The program used more stack than it was allowed to.",
    ),
    ("EI101", "The TableGen source could not be read."),
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
