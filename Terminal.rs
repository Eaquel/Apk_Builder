use crate::json::Writer;

pub const FENCE: &str = "```";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Text(String),
    Code(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub number: usize,
    pub heading: String,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub lead: String,
    pub entries: Vec<Entry>,
}

pub fn document(tag: &str) -> (&'static str, &'static str) {
    let documents = crate::terminal_speech::DOCUMENTS;
    let spoken = tag.split(['-', '_']).next().unwrap_or("");
    documents
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(tag))
        .or_else(|| {
            documents
                .iter()
                .find(|(known, _)| known.eq_ignore_ascii_case(spoken))
        })
        .or_else(|| documents.first())
        .map_or(("en", ""), |(known, text)| (*known, *text))
}

pub fn read(text: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.starts_with(FENCE) {
            fenced = !fenced;
            held.push(line);
            continue;
        }
        if !fenced {
            if let Some(title) = line.strip_prefix("@@ ") {
                close(&mut sections, &mut held);
                sections.push(Section {
                    title: title.trim().to_string(),
                    lead: String::new(),
                    entries: Vec::new(),
                });
                continue;
            }
            if let Some(lead) = line.strip_prefix("@: ") {
                if let Some(section) = sections.last_mut() {
                    section.lead = lead.trim().to_string();
                }
                continue;
            }
            if line.trim() == "---" {
                close(&mut sections, &mut held);
                continue;
            }
        }
        held.push(line);
    }
    close(&mut sections, &mut held);
    sections
}

fn close(sections: &mut [Section], held: &mut Vec<&str>) {
    let lines = std::mem::take(held);
    if let (Some(section), Some(entry)) = (sections.last_mut(), entry(&lines)) {
        section.entries.push(entry);
    }
}

fn entry(lines: &[&str]) -> Option<Entry> {
    let mut rest = lines.iter().skip_while(|line| line.trim().is_empty());
    let heading = rest.next()?.trim().to_string();
    let number = heading
        .split_once("- ")
        .and_then(|(said, _)| said.trim().parse().ok())
        .unwrap_or(0);
    let mut blocks = Vec::new();
    let mut prose: Vec<&str> = Vec::new();
    let mut code: Option<Vec<&str>> = None;
    for line in rest {
        if let Some(open) = code.as_mut() {
            if line.starts_with(FENCE) {
                blocks.push(Block::Code(open.join("\n")));
                code = None;
            } else {
                open.push(line);
            }
            continue;
        }
        if line.starts_with(FENCE) {
            said(&mut blocks, &mut prose);
            code = Some(Vec::new());
            continue;
        }
        if line.trim().is_empty() {
            said(&mut blocks, &mut prose);
            continue;
        }
        prose.push(line.trim_end());
    }
    said(&mut blocks, &mut prose);
    if let Some(open) = code {
        blocks.push(Block::Code(open.join("\n")));
    }
    Some(Entry {
        number,
        heading,
        blocks,
    })
}

fn said(blocks: &mut Vec<Block>, prose: &mut Vec<&str>) {
    if !prose.is_empty() {
        blocks.push(Block::Text(prose.join("\n")));
        prose.clear();
    }
}

pub fn write_json(language: &str, sections: &[Section], w: &mut Writer) {
    w.begin_object(None);
    w.field_str("language", language);
    w.begin_array(Some("sections"));
    for section in sections {
        w.begin_object(None);
        w.field_str("title", &section.title);
        w.field_str("lead", &section.lead);
        w.begin_array(Some("entries"));
        for entry in &section.entries {
            w.begin_object(None);
            w.field_u64("number", entry.number as u64);
            w.field_str("heading", &entry.heading);
            w.begin_array(Some("blocks"));
            for block in &entry.blocks {
                w.begin_object(None);
                match block {
                    Block::Text(body) => {
                        w.field_str("kind", "text");
                        w.field_str("body", body);
                    }
                    Block::Code(body) => {
                        w.field_str("kind", "code");
                        w.field_str("body", body);
                    }
                }
                w.end_object();
            }
            w.end_array();
            w.end_object();
        }
        w.end_array();
        w.end_object();
    }
    w.end_array();
    w.end_object();
}

pub fn docs_json(tag: &str) -> String {
    let (language, text) = document(tag);
    let mut w = Writer::new();
    write_json(language, &read(text), &mut w);
    w.finish()
}
