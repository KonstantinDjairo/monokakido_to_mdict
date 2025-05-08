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
    // Read entire file content
    let xml_content = std::fs::read_to_string(path)?;
    
    // Parse with quick_xml for accurate headword extraction
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    
    let mut headword = String::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                if e.name().as_ref() == b"div" {
                    // Handle attribute parsing errors properly
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        if attr.key.as_ref() == b"class" && attr.value.as_ref() == headword_tag.as_bytes() {
                            // Read text content properly including nested elements
                            headword = reader.read_text(e.name())?.trim().to_string();
                            break;
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => (),
        }
        buf.clear();
    }

    // Preserve exact HTML structure
    let html_content = xml_content
        .lines()
        .skip(1) // Skip XML declaration
        .collect::<Vec<_>>()
        .join("\n");

    if !headword.is_empty() {
        writeln!(writer, "{}", headword)?;
        writeln!(writer, "{}", html_content.trim())?;
        writeln!(writer, "</>")?;
    }

    Ok(())
}

// Main function remains unchanged
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
