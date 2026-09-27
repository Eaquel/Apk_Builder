use crate::compilers::java_fixer::{Confidence, Repair};
use crate::diag::Diagnostic;

pub const REPAIR_TITLES: &[&str] = &[
    "Close the comment that was left open.",
    "Close the text that was left open.",
    "Close the block that was left open.",
    "Close the string template that was left open.",
    "Write the mark the language expects here.",
    "Separate the two statements with a semicolon.",
    "Import the class this file names.",
    "Close the name that was opened with a backtick.",
];

fn context_value(refusal: &Diagnostic, label: &str) -> Option<String> {
    let head = format!("{label}: ");
    refusal
        .context
        .iter()
        .find_map(|line| line.strip_prefix(head.as_str()))
        .map(|held| held.trim().to_string())
}

fn source_of(root: &str, named: &str) -> Option<String> {
    if named.is_empty() {
        return None;
    }
    std::fs::read_to_string(std::path::Path::new(root).join(named)).ok()
}

fn line_at(source: &str, line: u32) -> Option<String> {
    if line == 0 {
        return None;
    }
    source.lines().nth(line as usize - 1).map(str::to_string)
}

fn mended(
    refusal: &Diagnostic,
    named: &str,
    line: u32,
    title: &str,
    before: String,
    after: String,
    confidence: Confidence,
) -> Repair {
    Repair {
        code: refusal.code.clone(),
        path: named.to_string(),
        title: title.to_string(),
        line,
        before,
        after,
        confidence,
    }
}

fn closing_mark(code: &str) -> Option<(&'static str, &'static str)> {
    match code {
        "EK001" => Some(("*/", "Close the comment that was left open.")),
        "EK104" | "EK106" | "EK112" => Some(("}", "Close the block that was left open.")),
        _ => None,
    }
}

fn sources_under(root: &str) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut waiting: Vec<std::path::PathBuf> = ["Kotlin", "Java"]
        .iter()
        .map(|folder| std::path::Path::new(root).join(folder))
        .collect();
    while let Some(here) = waiting.pop() {
        let Ok(entries) = std::fs::read_dir(&here) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                waiting.push(path);
            } else if path
                .extension()
                .is_some_and(|held| held == "kt" || held == "java")
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn declared_in(root: &str, named: &str) -> Option<String> {
    let shapes = [
        format!("class {named}"),
        format!("interface {named}"),
        format!("object {named}"),
        format!("enum class {named}"),
        format!("typealias {named}"),
    ];
    for path in sources_under(root) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let holds = shapes.iter().any(|wanted| {
            text.match_indices(wanted.as_str()).any(|(at, _)| {
                let after = at + wanted.len();
                text.as_bytes()
                    .get(after)
                    .is_none_or(|held| !held.is_ascii_alphanumeric() && *held != b'_')
            })
        });
        if !holds {
            continue;
        }
        for line in text.lines() {
            let Some(rest) = line.trim().strip_prefix("package ") else {
                continue;
            };
            let held = rest.trim().trim_end_matches(';').trim();
            if held.is_empty() {
                break;
            }
            return Some(format!("{held}.{named}"));
        }
    }
    None
}

fn known_as(root: &str, named: &str) -> Option<String> {
    let found = crate::symbols::matching(root, named).ok()?;
    let mut exact: Vec<&crate::symbols::Named> =
        found.iter().filter(|one| one.simple == named).collect();
    exact.sort_by(|left, right| {
        right
            .rank
            .cmp(&left.rank)
            .then_with(|| left.qualified.len().cmp(&right.qualified.len()))
    });
    let best = exact.first()?;
    if best.qualified.contains('.') {
        Some(best.qualified.clone())
    } else {
        None
    }
}

fn import_line(source: &str) -> (u32, String) {
    let mut at = 0u32;
    let mut last_import = 0u32;
    let mut package_at = 0u32;
    for line in source.lines() {
        at += 1;
        let held = line.trim();
        if held.starts_with("package ") {
            package_at = at;
        }
        if held.starts_with("import ") {
            last_import = at;
        }
    }
    let anchor = if last_import > 0 {
        last_import
    } else {
        package_at
    };
    if anchor == 0 {
        return (0, String::new());
    }
    let text = source
        .lines()
        .nth(anchor as usize - 1)
        .unwrap_or_default()
        .to_string();
    (anchor, text)
}

pub fn repairs_for(root: &str, refusal: &Diagnostic) -> Vec<Repair> {
    let mut out = Vec::new();
    let Some(place) = refusal.location.as_ref() else {
        return out;
    };
    let Some(source) = source_of(root, &place.file) else {
        return out;
    };
    let line = place.line;
    let named = place.file.as_str();

    if let Some((mark, title)) = closing_mark(refusal.code.as_str()) {
        let ends = source.lines().count() as u32;
        let Some(last) = line_at(&source, ends) else {
            return out;
        };
        out.push(mended(
            refusal,
            named,
            ends,
            title,
            last.clone(),
            format!("{last}\n{mark}"),
            Confidence::Certain,
        ));
        return out;
    }

    let Some(held) = line_at(&source, line) else {
        return out;
    };

    match refusal.code.as_str() {
        "EK002" => out.push(mended(
            refusal,
            named,
            line,
            "Close the name that was opened with a backtick.",
            held.clone(),
            format!("{held}`"),
            Confidence::Likely,
        )),
        "EK009" => out.push(mended(
            refusal,
            named,
            line,
            "Close the text that was left open.",
            held.clone(),
            format!("{held}\""),
            Confidence::Likely,
        )),
        "EK008" => out.push(mended(
            refusal,
            named,
            line,
            "Close the string template that was left open.",
            held.clone(),
            format!("{held}}}"),
            Confidence::Likely,
        )),
        "EK100" => {
            let Some(mark) = context_value(refusal, "Expected") else {
                return out;
            };
            if held.trim_end().ends_with(mark.as_str()) {
                return out;
            }
            out.push(mended(
                refusal,
                named,
                line,
                "Write the mark the language expects here.",
                held.clone(),
                format!("{}{}", held.trim_end(), mark),
                Confidence::Likely,
            ));
        }
        "EK107" => {
            let column = place.column as usize;
            let cut = held
                .char_indices()
                .nth(column.saturating_sub(1))
                .map(|(at, _)| at)
                .unwrap_or(held.len());
            let (left, right) = held.split_at(cut);
            if left.trim().is_empty() || right.trim().is_empty() {
                return out;
            }
            out.push(mended(
                refusal,
                named,
                line,
                "Separate the two statements with a semicolon.",
                held.clone(),
                format!("{}; {}", left.trim_end(), right.trim_start()),
                Confidence::Likely,
            ));
        }
        "EK200" | "EK210" => {
            let wanted = context_value(refusal, "Type")
                .filter(|_| refusal.code == "EK200")
                .or_else(|| context_value(refusal, "Name"));
            let Some(wanted) = wanted else {
                return out;
            };
            let wanted = wanted
                .split(['<', '?', ' '])
                .next()
                .unwrap_or_default()
                .to_string();
            if wanted.is_empty()
                || !wanted.starts_with(|one: char| one.is_ascii_uppercase())
                || source.contains(&format!("import {wanted}"))
                || source.contains(&format!(".{wanted}\n"))
            {
                return out;
            }
            let Some(full) = declared_in(root, &wanted).or_else(|| known_as(root, &wanted)) else {
                return out;
            };
            let (at, before) = import_line(&source);
            if at == 0 {
                return out;
            }
            out.push(mended(
                refusal,
                named,
                at,
                "Import the class this file names.",
                before.clone(),
                format!("{before}\nimport {full}"),
                Confidence::Likely,
            ));
        }
        _ => {}
    }
    out
}
