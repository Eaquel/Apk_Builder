use crate::diag::{Diagnostic, Severity};
use crate::FailureClass;

const ORIGIN: &str = "compilers.java.fixer";

pub(crate) fn fail(code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        code,
        Severity::Error,
        FailureClass::UserError,
        ORIGIN,
        message,
    )
}

pub const REPAIR_TITLES: &[&str] = &[
    "Declare the missing resource in the project's own values folder.",
    "Remove the attribute that names a resource this project does not hold.",
    "Set the manifest's package from the package its sources declare.",
    "Write the activity's name out in full.",
    "Close the comment that was left open.",
    "Close the text that was left open.",
    "Close the block that was left open.",
    "Close the list of type arguments.",
    "Write the mark the language expects here.",
    "Write the number without the zero it leads with.",
    "Mark the number as a long.",
    "Give the method a body of its own.",
    "Take out the arm that is written twice.",
    "Import the class this file names.",
    "Break the line after the three quotes a text block opens with.",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    Certain,
    Likely,
    Risky,
}

impl Confidence {
    pub const fn as_str(self) -> &'static str {
        match self {
            Confidence::Certain => "CERTAIN",
            Confidence::Likely => "LIKELY",
            Confidence::Risky => "RISKY",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repair {
    pub code: String,
    pub path: String,
    pub title: String,
    pub line: u32,
    pub before: String,
    pub after: String,
    pub confidence: Confidence,
}

impl Repair {
    pub fn spoken_title(&self) -> String {
        crate::speech::sentence(&self.title)
    }

    pub fn from_json(held: &crate::json::Json) -> Option<Repair> {
        let word = |key: &str| {
            held.get(key)
                .and_then(|one| one.text())
                .unwrap_or_default()
                .to_string()
        };
        let code = word("code");
        let path = word("path");
        if code.is_empty() || path.is_empty() {
            return None;
        }
        let confidence = match word("confidence").as_str() {
            "CERTAIN" => Confidence::Certain,
            "RISKY" => Confidence::Risky,
            _ => Confidence::Likely,
        };
        Some(Repair {
            code,
            path,
            title: word("english"),
            line: held
                .get("line")
                .and_then(|one| one.integer())
                .unwrap_or_default()
                .max(0) as u32,
            before: word("before"),
            after: word("after"),
            confidence,
        })
    }

    pub fn write_json(&self, w: &mut crate::json::Writer) {
        w.begin_object(None);
        w.field_str("code", &self.code);
        w.field_str("path", &self.path);
        w.field_str("title", &self.spoken_title());
        w.field_str("english", &self.title);
        w.field_u64("line", u64::from(self.line));
        w.field_str("confidence", self.confidence.as_str());
        w.field_str("before", &self.before);
        w.field_str("after", &self.after);
        w.end_object();
    }
}

fn context_value(refusal: &Diagnostic, label: &str) -> Option<String> {
    let head = format!("{label}: ");
    refusal
        .context
        .iter()
        .find_map(|line| line.strip_prefix(head.as_str()))
        .map(|held| held.trim().to_string())
}

fn manifest_at(root: &str) -> std::path::PathBuf {
    std::path::Path::new(root).join("AndroidManifest.xml")
}

fn attribute_text(source: &str, name: &str) -> Option<String> {
    let mut at = 0usize;
    let held = source.as_bytes();
    while let Some(found) = source[at..].find(name) {
        let start = at + found;
        let before = start.checked_sub(1).map(|one| held[one]);
        if !matches!(before, Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')) {
            at = start + name.len();
            continue;
        }
        let rest = &source[start + name.len()..];
        let trimmed = rest.trim_start();
        if !trimmed.starts_with('=') {
            at = start + name.len();
            continue;
        }
        let after_equals = &trimmed[1..];
        let quoted = after_equals.trim_start();
        let mark = quoted.chars().next()?;
        if mark != '"' && mark != '\'' {
            return None;
        }
        let closing = quoted[1..].find(mark)? + 1;
        let consumed =
            (rest.len() - trimmed.len()) + 1 + (after_equals.len() - quoted.len()) + closing + 1;
        return Some(source[start..start + name.len() + consumed].to_string());
    }
    None
}

fn declared_package(root: &str) -> Option<String> {
    let sources = std::path::Path::new(root).join("Java");
    let mut found: Option<String> = None;
    let mut waiting = vec![sources];
    while let Some(here) = waiting.pop() {
        let Ok(entries) = std::fs::read_dir(&here) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                waiting.push(path);
                continue;
            }
            if path.extension().is_none_or(|held| held != "java") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in text.lines() {
                let line = line.trim();
                let Some(rest) = line.strip_prefix("package ") else {
                    continue;
                };
                let named = rest.trim().trim_end_matches(';').trim();
                if named.is_empty() {
                    continue;
                }
                if found.is_none() {
                    found = Some(named.to_string());
                }
                break;
            }
        }
    }
    found
}

fn values_folder(root: &str) -> std::path::PathBuf {
    std::path::Path::new(root).join("Res").join("values")
}

fn declaration_for(kind: &str, name: &str) -> Option<String> {
    let held = match kind {
        "style" => format!(
            "    <style name=\"{name}\" parent=\"android:Theme.Material.Light.NoActionBar\" />"
        ),
        "string" => format!("    <string name=\"{name}\">{name}</string>"),
        "color" => format!("    <color name=\"{name}\">#FF000000</color>"),
        "bool" => format!("    <bool name=\"{name}\">false</bool>"),
        "integer" => format!("    <integer name=\"{name}\">0</integer>"),
        "dimen" => format!("    <dimen name=\"{name}\">0dp</dimen>"),
        _ => return None,
    };
    Some(held)
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
        "EJ001" => Some(("*/", "Close the comment that was left open.")),
        "EJ105" | "EJ107" | "EJ108" | "EJ110" => Some(("}", "Close the block that was left open.")),
        _ => None,
    }
}

fn leading_zero_gone(line: &str) -> Option<String> {
    let held = line.as_bytes();
    let mut at = 0usize;
    while at < held.len() {
        if !held[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let start = at;
        while at < held.len() && held[at].is_ascii_digit() {
            at += 1;
        }
        let number = &line[start..at];
        if number.len() < 2 || !number.starts_with('0') {
            continue;
        }
        if start > 0 && (held[start - 1].is_ascii_alphanumeric() || held[start - 1] == b'.') {
            continue;
        }
        let bare = number.trim_start_matches('0');
        let bare = if bare.is_empty() { "0" } else { bare };
        return Some(format!("{}{}{}", &line[..start], bare, &line[at..]));
    }
    None
}

fn number_made_long(line: &str) -> Option<String> {
    let held = line.as_bytes();
    let mut at = 0usize;
    while at < held.len() {
        if !held[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let start = at;
        while at < held.len() && held[at].is_ascii_digit() {
            at += 1;
        }
        if at < held.len() && matches!(held[at], b'L' | b'l' | b'.' | b'e' | b'E') {
            continue;
        }
        if start > 0 && (held[start - 1].is_ascii_alphabetic() || held[start - 1] == b'_') {
            continue;
        }
        return Some(format!(
            "{}{}L{}",
            &line[..start],
            &line[start..at],
            &line[at..]
        ));
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

fn declared_in(root: &str, named: &str) -> Option<String> {
    let sources = std::path::Path::new(root).join("Java");
    let mut waiting = vec![sources];
    let wanted_class = format!("class {named}");
    let wanted_interface = format!("interface {named}");
    let wanted_enum = format!("enum {named}");
    let wanted_record = format!("record {named}");
    while let Some(here) = waiting.pop() {
        let Ok(entries) = std::fs::read_dir(&here) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                waiting.push(path);
                continue;
            }
            if path.extension().is_none_or(|held| held != "java") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let holds = [
                &wanted_class,
                &wanted_interface,
                &wanted_enum,
                &wanted_record,
            ]
            .iter()
            .any(|wanted| {
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
                let line = line.trim();
                let Some(rest) = line.strip_prefix("package ") else {
                    continue;
                };
                let held = rest.trim().trim_end_matches(';').trim();
                if held.is_empty() {
                    break;
                }
                return Some(format!("{held}.{named}"));
            }
        }
    }
    None
}

fn import_line(source: &str) -> (u32, String, String) {
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
        return (0, String::new(), String::new());
    }
    let text = source
        .lines()
        .nth(anchor as usize - 1)
        .unwrap_or_default()
        .to_string();
    (anchor, text.clone(), text)
}

fn java_repairs(root: &str, refusal: &Diagnostic) -> Vec<Repair> {
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
        "EJ005" => out.push(mended(
            refusal,
            named,
            line,
            "Close the text that was left open.",
            held.clone(),
            format!("{held}\""),
            Confidence::Likely,
        )),
        "EJ011" => {
            if let Some(at) = held.find("\"\"\"") {
                let cut = at + 3;
                if held[cut..].trim().is_empty() {
                    return out;
                }
                out.push(mended(
                    refusal,
                    named,
                    line,
                    "Break the line after the three quotes a text block opens with.",
                    held.clone(),
                    format!("{}\n{}", &held[..cut], &held[cut..]),
                    Confidence::Certain,
                ));
            }
        }
        "EJ115" => out.push(mended(
            refusal,
            named,
            line,
            "Close the list of type arguments.",
            held.clone(),
            format!("{held}>"),
            Confidence::Likely,
        )),
        "EJ100" => {
            let mark = context_value(refusal, "Mark").unwrap_or_else(|| ";".to_string());
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
        "EJ007" => {
            if let Some(fixed) = leading_zero_gone(&held) {
                out.push(mended(
                    refusal,
                    named,
                    line,
                    "Write the number without the zero it leads with.",
                    held.clone(),
                    fixed,
                    Confidence::Certain,
                ));
            }
        }
        "EJ008" | "EJ010" => {
            if let Some(fixed) = number_made_long(&held) {
                out.push(mended(
                    refusal,
                    named,
                    line,
                    "Mark the number as a long.",
                    held.clone(),
                    fixed,
                    Confidence::Likely,
                ));
            }
        }
        "EJ106" => {
            let trimmed = held.trim_end();
            if !trimmed.ends_with(';') {
                return out;
            }
            out.push(mended(
                refusal,
                named,
                line,
                "Give the method a body of its own.",
                held.clone(),
                format!("{} {{\n    }}", trimmed.trim_end_matches(';')),
                Confidence::Likely,
            ));
        }
        "EJ113" | "EJ240" => out.push(mended(
            refusal,
            named,
            line,
            "Take out the arm that is written twice.",
            held.clone(),
            String::new(),
            Confidence::Risky,
        )),
        "EJ200" | "EJ211" => {
            let Some(wanted) = context_value(refusal, "Name") else {
                return out;
            };
            if source.contains(&format!("import {wanted};"))
                || source.contains(&format!(".{wanted};"))
            {
                return out;
            }
            let Some(full) = declared_in(root, &wanted).or_else(|| known_as(root, &wanted)) else {
                return out;
            };
            let (at, before, after) = import_line(&source);
            if at == 0 {
                return out;
            }
            out.push(mended(
                refusal,
                named,
                at,
                "Import the class this file names.",
                before,
                format!("{after}\nimport {full};"),
                Confidence::Likely,
            ));
        }
        _ => {}
    }
    out
}

pub fn repairs_for(root: &str, refusal: &Diagnostic) -> Vec<Repair> {
    if refusal.code.starts_with("EJ") {
        return java_repairs(root, refusal);
    }
    let mut out = Vec::new();
    match refusal.code.as_str() {
        "EB042" | "EB045" => {
            let (Some(attribute), Some(value)) = (
                context_value(refusal, "Attribute"),
                context_value(refusal, "Value"),
            ) else {
                return out;
            };
            let Ok(source) = std::fs::read_to_string(manifest_at(root)) else {
                return out;
            };
            if let Some(written) = attribute_text(&source, &attribute) {
                out.push(Repair {
                    line: 0,
                    confidence: Confidence::Likely,
                    code: refusal.code.clone(),
                    path: "AndroidManifest.xml".to_string(),
                    title: "Remove the attribute that names a resource this project does not hold."
                        .to_string(),
                    before: written,
                    after: String::new(),
                });
            }
            let named = value.trim_start_matches('@');
            if let Some((kind, name)) = named.split_once('/') {
                let kind = kind.rsplit(':').next().unwrap_or(kind);
                if let Some(line) = declaration_for(kind, name) {
                    let file = format!("Res/values/{kind}s.xml");
                    let path = values_folder(root).join(format!("{kind}s.xml"));
                    let (before, after) = match std::fs::read_to_string(&path) {
                        Ok(held) if held.contains("</resources>") => (
                            "</resources>".to_string(),
                            format!("{line}\n</resources>"),
                        ),
                        _ => (
                            String::new(),
                            format!(
                                "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n{line}\n</resources>\n"
                            ),
                        ),
                    };
                    out.push(Repair {
                        line: 0,
                        confidence: Confidence::Likely,
                        code: refusal.code.clone(),
                        path: file,
                        title: "Declare the missing resource in the project's own values folder."
                            .to_string(),
                        before,
                        after,
                    });
                }
            }
        }
        "EB013" | "EB014" => {
            let Ok(source) = std::fs::read_to_string(manifest_at(root)) else {
                return out;
            };
            let Some(package) = declared_package(root) else {
                return out;
            };
            if attribute_text(&source, "package").is_none() {
                if let Some(found) = source.find("<manifest") {
                    let head = &source[found..];
                    let Some(stop) = head.find('>') else {
                        return out;
                    };
                    let opening = &head[..stop];
                    out.push(Repair {
                        line: 0,
                        confidence: Confidence::Likely,
                        code: refusal.code.clone(),
                        path: "AndroidManifest.xml".to_string(),
                        title: "Set the manifest's package from the package its sources declare."
                            .to_string(),
                        before: opening.to_string(),
                        after: format!("{opening}\n    package=\"{package}\""),
                    });
                }
            }
            if let Some(written) = attribute_text(&source, "android:name") {
                if let Some(opened) = written.find('"') {
                    let value = &written[opened + 1..written.len() - 1];
                    if let Some(rest) = value.strip_prefix('.') {
                        out.push(Repair {
                            line: 0,
                            confidence: Confidence::Likely,
                            code: refusal.code.clone(),
                            path: "AndroidManifest.xml".to_string(),
                            title: "Write the activity's name out in full.".to_string(),
                            before: written.clone(),
                            after: format!("android:name=\"{package}.{rest}\""),
                        });
                    }
                }
            }
        }
        _ => {}
    }
    out
}

pub fn apply(root: &str, repair: &Repair) -> Result<(), Diagnostic> {
    let named = std::path::Path::new(&repair.path);
    let wanders = named.components().any(|one| {
        matches!(
            one,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    });
    if wanders || repair.path.is_empty() {
        return Err(fail(
            "EF001",
            "A repair names a file that is not in this project.",
        )
        .with_context(format!("Path: {}", repair.path)));
    }
    let path = std::path::Path::new(root).join(named);
    if repair.line == 0 && repair.before.is_empty() {
        if let Some(folder) = path.parent() {
            std::fs::create_dir_all(folder).map_err(|why| {
                fail("EF003", "A repair could not be written.")
                    .with_context(format!("Path: {}", repair.path))
                    .with_context(format!("Cause: {why}"))
            })?;
        }
        std::fs::write(&path, &repair.after).map_err(|why| {
            fail("EF003", "A repair could not be written.")
                .with_context(format!("Path: {}", repair.path))
                .with_context(format!("Cause: {why}"))
        })?;
        return Ok(());
    }
    let source = std::fs::read_to_string(&path).map_err(|why| {
        fail("EF003", "A repair could not be written.")
            .with_context(format!("Path: {}", repair.path))
            .with_context(format!("Cause: {why}"))
    })?;
    let written = if repair.line > 0 {
        let mut held: Vec<&str> = source.lines().collect();
        let at = repair.line as usize - 1;
        if held
            .get(at)
            .map(|one| *one != repair.before)
            .unwrap_or(true)
        {
            return Err(fail(
                "EF002",
                "A repair no longer matches the file it was made for.",
            )
            .with_context(format!("Path: {}", repair.path))
            .with_context(format!("Line {}", repair.line)));
        }
        if repair.after.is_empty() {
            held.remove(at);
        } else {
            held[at] = repair.after.as_str();
        }
        let mut out = held.join("\n");
        if source.ends_with('\n') {
            out.push('\n');
        }
        out
    } else {
        let Some(at) = source.find(&repair.before) else {
            return Err(fail(
                "EF002",
                "A repair no longer matches the file it was made for.",
            )
            .with_context(format!("Path: {}", repair.path)));
        };
        let mut out = String::with_capacity(source.len() + repair.after.len());
        out.push_str(&source[..at]);
        out.push_str(&repair.after);
        out.push_str(&source[at + repair.before.len()..]);
        out
    };
    std::fs::write(&path, written).map_err(|why| {
        fail("EF003", "A repair could not be written.")
            .with_context(format!("Path: {}", repair.path))
            .with_context(format!("Cause: {why}"))
    })
}

pub fn repairs_for_all(root: &str, refusals: &[Diagnostic]) -> Vec<Repair> {
    let mut out: Vec<Repair> = Vec::new();
    for refusal in refusals {
        for made in repairs_for(root, refusal) {
            let clashes = out
                .iter()
                .any(|held| held.path == made.path && held.line == made.line && made.line > 0);
            if !clashes {
                out.push(made);
            }
        }
    }
    out.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| right.line.cmp(&left.line))
    });
    out
}

pub fn write_json(repairs: &[Repair], w: &mut crate::json::Writer, key: &str) {
    w.begin_array(Some(key));
    for made in repairs {
        made.write_json(w);
    }
    w.end_array();
}
