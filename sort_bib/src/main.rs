use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

fn main() -> Result<()> {
    if env::args().count() != 2 {
        eprintln!("Missing input file");
        std::process::exit(1);
    }
    let filename = env::args().nth(1).unwrap();
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut citations: HashMap<String, Vec<String>> = HashMap::new();
    let mut key: String = String::from("");
    for line in reader.lines() {
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('@') {
            let start = line.chars().position(|c| c == '{').unwrap();
            key = line[start + 1..line.len() - 1].to_string();
        }
        citations
            .entry(key.clone())
            .or_default()
            .push(line.to_string());
    }
    let mut names: Vec<&str> = citations.keys().map(|s| s.as_str()).collect();
    names.sort();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("sorted.bib")?;
    for name in names.iter() {
        let entry = citations.get(*name).unwrap();
        for line in entry.iter() {
            writeln!(file, "{}", line)?;
        }
    }
    Ok(())
}
