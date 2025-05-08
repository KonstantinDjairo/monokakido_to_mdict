// Copyright © 2025 Hashirama Senju

// TODO: fix writing order, the results are being write in the wrong order, 
// and it's not processing all files for some reason
use quick_xml::events::Event;
use quick_xml::Reader;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use walkdir::WalkDir;

fn process_xml_file(
    path: &Path,
    headword_tag: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let xml_content = std::fs::read_to_string(path)?;
    
    let mut reader = Reader::from_str(&xml_content);
    reader.trim_text(true);
    
    let mut headword = String::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                if e.name().as_ref() == b"div" {
                    for attr in e.attributes().filter_map(|a| a.ok()) {
                        if attr.key.as_ref() == b"class" && attr.value.as_ref() == headword_tag.as_bytes() {
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

    let html_content = xml_content
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");

    Ok((headword, html_content))
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

    // Collect all XML files first
    let files: Vec<PathBuf> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "xml").unwrap_or(false))
        .map(|e| e.path().to_owned())
        .collect();

    let total_files = files.len();
    let progress_interval = match total_files {
        0 => 1,
        1..=100 => 1,
        101..=10_000 => total_files / 100,
        _ => total_files / 200,
    }.max(1);

    println!("Processing {} files (updating progress every {} entries)...", 
        total_files, progress_interval);

    let output_file = File::create("dictionary.mdict")?;
    let writer = Arc::new(Mutex::new(BufWriter::new(output_file)));
    
    // Create work queue
    let queue = Arc::new(Mutex::new(files));
    let (tx, rx) = mpsc::channel();

    // Determine thread count
    let num_threads = num_cpus::get() * 2;
    
    // Spawn worker threads
    let mut handles = vec![];
    for _ in 0..num_threads {
        let queue = Arc::clone(&queue);
        let tx = tx.clone();
        let headword_tag = headword_tag.to_string();
        
        let handle = thread::spawn(move || {
            loop {
                let path = {
                    let mut q = queue.lock().unwrap();
                    q.pop()
                };
                
                let path = match path {
                    Some(p) => p,
                    None => break,
                };

                match process_xml_file(&path, &headword_tag) {
                    Ok((headword, html)) => tx.send((headword, html)).unwrap(),
                    Err(e) => eprintln!("Error processing {}: {}", path.display(), e),
                }
            }
        });
        handles.push(handle);
    }

    // Spawn writer thread
    let writer_handle = thread::spawn(move || {
        let mut count = 0;
        while let Ok((headword, html)) = rx.recv() {
            let mut writer = writer.lock().unwrap();
            writeln!(writer, "{}", headword).unwrap();
            writeln!(writer, "{}", html.trim()).unwrap();
            writeln!(writer, "</>").unwrap();
            
            count += 1;
            if count % progress_interval == 0 {
                let percentage = (count as f32 / total_files as f32 * 100.0).round();
                println!("Progress: {}% ({}/{} entries)", percentage, count, total_files);
            }
        }
        println!("Final count: {} entries processed", count);
    });

    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }
    writer_handle.join().unwrap();

    println!("Completed processing all files");
    Ok(())
}

