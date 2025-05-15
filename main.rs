// Copyright © 2025 Hashirama Senju
use quick_xml::events::Event;
use quick_xml::Reader;
use rayon::prelude::*;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Result for one processed file
struct ProcessedEntry {
    headword: String,
    content: String,
}

fn process_xml_file(
    path: &Path,
    headword_tag: &str,
) -> Result<Option<ProcessedEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let xml_content = std::fs::read_to_string(path)?;

    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);

    let mut headword = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let is_tag = e.name().as_ref() == headword_tag.as_bytes();
                let class_matches = e
                    .attributes()
                    .filter_map(Result::ok)
                    .any(|attr| attr.key.as_ref() == b"class"
                        && attr.value.as_ref() == headword_tag.as_bytes());

                if is_tag || class_matches {
                    // FIXME: we arent extracting the headword correctly, 
                    //        because it should not contain any html tag along with it.
                    headword = reader.read_text(e.name())?.trim().to_string();
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    if headword.is_empty() {
        return Ok(None);
    }

    let html_content = xml_content
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    Ok(Some(ProcessedEntry {
        headword,
        content: html_content.trim().to_string(),
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

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

    let entries: Vec<PathBuf> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| p.extension().map(|e| e == "xml").unwrap_or(false))
        .collect();

    // Parallel processing
    let processed: Vec<_> = entries
    .par_iter()
    .filter_map(|path| match process_xml_file(path, headword_tag) {
        Ok(Some(entry)) => Some(Ok::<_, Box<dyn std::error::Error + Send + Sync>>(entry)),
        Ok(None) => None,
        Err(e) => {
            eprintln!("Error processing {}: {}", path.display(), e);
            None
        }
    })
    .collect::<Result<_, _>>()?;


    // Write sequentially
    let output_file = File::create("dictionary.mdict")?;
    let mut writer = BufWriter::new(output_file);

    for entry in processed {
        writeln!(writer, "{}", entry.headword)?;
        writeln!(writer, "{}", entry.content)?;
        writeln!(writer, "</>")?;
    }

    writer.flush()?;
    Ok(())
}
