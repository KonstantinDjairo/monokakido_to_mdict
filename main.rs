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
    headword_tag: &str, // pass "headword" here
) -> Result<Option<ProcessedEntry>, Box<dyn std::error::Error + Send + Sync>> {
    
    let xml = std::fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);

  
    let mut buf = Vec::new();
    let mut headword = String::new();
    let mut in_headword_div = false;

 
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) if e.name().as_ref() == b"div" => {
                // check class="headword"
                let is_headword_div = e
                    .attributes()
                    .filter_map(Result::ok)
                    .any(|attr| attr.key.as_ref() == b"class"
                        && attr.value.as_ref() == headword_tag.as_bytes());
                if is_headword_div {
                    in_headword_div = true;
                }
            }

            Event::Text(e) if in_headword_div => {
                // append only the text, unescaped and trimmed
                headword.push_str(e.unescape()?.trim());
            }

            Event::End(e) if in_headword_div && e.name().as_ref() == b"div" => {
                // end of headword div
                break;
            }

            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    if headword.is_empty() {
        return Ok(None);
    }

    let html_body = xml
        .lines()
        .skip(1)                     // drop "<?xml...?>" line
        .collect::<Vec<_>>()
        .join("")                    // collapse into one line
        .trim()                      // trim leading/trailing whitespace
        .to_string();

    Ok(Some(ProcessedEntry {
        headword,
        content: html_body,
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
