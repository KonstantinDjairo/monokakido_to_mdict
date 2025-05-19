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
    let xml = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut headword = String::new();
    let mut in_headword = false;
    let tag_bytes = headword_tag.as_bytes();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let is_tag = e.name().as_ref() == tag_bytes;
                let class_matches = e
                    .attributes()
                    .filter_map(Result::ok)
                    .any(|attr| attr.key.as_ref() == b"class" && attr.value.as_ref() == tag_bytes);

                if is_tag || class_matches {
                    in_headword = true;
                }
            }

Event::Text(e) if in_headword => {
    // 1) Unescape into a Cow<'_, str> and then into an owned String
    let decoded_owned: String = e.unescape()?.into_owned();
    // 2) Trim whitespace and take only the first word
    if let Some(first) = decoded_owned
        .trim()
        .split_whitespace()
        .next()
    {
        headword.push_str(first);
        // stop collecting after first word
        in_headword = false;
    }
}



            Event::End(ref e) if in_headword && e.name().as_ref() == tag_bytes => {
                // Done with headword
                break;
            }

            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    // Skip files without a headword
    if headword.is_empty() {
        return Ok(None);
    }

    // Rebuild the rest of the file as one line (skip XML declaration)
    let content = xml
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    Ok(Some(ProcessedEntry { headword, content }))
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
