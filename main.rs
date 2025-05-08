// Copyright © 2025 Hashirama Senju
use quick_xml::events::Event;
use quick_xml::Reader;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use walkdir::WalkDir;

fn process_xml_file(
    path: &Path,
    writer: &mut BufWriter<File>,
    headword_tag: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = Reader::from_file(path)?;
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut in_entry_div = false;
    let mut in_headword_div = false;
    let mut entry_content = String::new();
    let mut headword = String::new();
    let mut depth = 0;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                if e.name().as_ref() == b"div" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"class" {
                            if attr.value.as_ref() == headword_tag.as_bytes() {
                                in_headword_div = true;
                            }
                            if attr.value.as_ref() == b"entry" {
                                in_entry_div = true;
                                entry_content.push_str(&String::from_utf8_lossy(&e.to_vec()));
                            }
                        }
                    }
                }
                
                if in_entry_div && !in_headword_div {
                    entry_content.push_str(&String::from_utf8_lossy(&e.to_vec()));
                }
                if in_entry_div {
                    depth += 1;
                }
            },
            Event::Text(e) => {
                if in_headword_div {
                    headword = e.unescape()?.into_owned();
                } else if in_entry_div {
                    entry_content.push_str(&e.unescape()?);
                }
            },
            Event::End(e) => {
                if in_entry_div {
                    if e.name().as_ref() == b"div" {
                        depth -= 1;
                        if depth == 0 {
                            in_entry_div = false;
                        } else if in_headword_div {
                            in_headword_div = false;
                        }
                    }
                    entry_content.push_str(&String::from_utf8_lossy(&e.to_vec()));
                }
            },
            Event::Eof => break,
            _ => (),
        }
    }

    if !headword.is_empty() && !entry_content.is_empty() {
        writeln!(writer, "{}", headword.trim())?;
        writeln!(writer, "{}", entry_content.trim())?;
        writeln!(writer, "</>")?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let (headword_tag, input_dir) = match args.as_slice() {
        [_, flag, tag, dir] if flag == "--headword-tag" => (tag.as_str(), dir),
        [_, dir] => ("mida", dir),
        _ => {
            eprintln!("Usage: {} [--headword-tag TAG] INPUT_DIR", args[0]);
            eprintln!("Default headword tag: mida");
            std::process::exit(1);
        }
    };

    let output_file = File::create("dictionary.mdict")?;
    let mut writer = BufWriter::new(output_file);

    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "xml").unwrap_or(false) {
            if let Err(e) = process_xml_file(path, &mut writer, headword_tag) {
                eprintln!("Error processing {}: {}", path.display(), e);
            }
        }
    }

    writer.flush()?;
    Ok(())
}
