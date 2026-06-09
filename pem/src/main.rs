use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::{env, fs, path::Path};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    List,
    Add { name: String, path: String },
    Amend { name: String, path: String },
    Get { name: String },
}

fn load_envs(fpath: &Path, record: &mut HashMap<String, String>) -> Result<()> {
    let file = fs::File::open(fpath)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let (name, path) = line.split_once(" ").unwrap();
        record.insert(name.trim().to_string(), path.trim().to_string());
    }
    Ok(())
}

fn save_envs(fpath: &Path, record: &HashMap<String, String>) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fpath)?;
    for (name, path) in record.iter() {
        writeln!(file, "{} {}", name, path)?;
    }
    Ok(())
}

fn add_environment(
    fpath: &Path,
    record: &mut HashMap<String, String>,
    env_name: &String,
    env_path: &String,
) -> Result<()> {
    let path = Path::new(env_path).join("bin").join("activate");
    if !path.is_file() {
        return Err(anyhow!("Not a valid python environment"));
    }
    record.insert(env_name.to_string(), env_path.to_string());
    save_envs(fpath, record)?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let home_dir = env::home_dir().expect("Could not find home directory");
    let pem_dir = home_dir.join(".local").join("pem");
    if !pem_dir.is_dir() {
        fs::create_dir_all(&pem_dir)?;
    }
    let pem_file = pem_dir.join("envs");
    let mut record: HashMap<String, String> = HashMap::new();
    if pem_file.is_file() {
        load_envs(pem_file.as_path(), &mut record)?;
    }
    match &cli.command {
        Commands::List => {
            if record.is_empty() {
                println!("No virtual environment saved");
                return Ok(());
            }
            for (name, path) in record.iter() {
                println!("{:<10} {}", name.bold(), path);
            }
        }
        Commands::Amend { name, path } => {
            if !record.contains_key(name) {
                let msg = format!("No environment with name '{}' found", name);
                return Err(anyhow!(msg));
            }
            add_environment(pem_file.as_path(), &mut record, name, path)?;
        }
        Commands::Add { name, path } => {
            if record.contains_key(name) {
                let msg = format!("Environment with name '{}' already added", name);
                return Err(anyhow!(msg));
            }
            add_environment(pem_file.as_path(), &mut record, name, path)?;
        }
        Commands::Get { name } => match record.get(name) {
            Some(path) => println!("{}/bin/activate", path),
            None => {
                let msg = format!("Environment {} is not known", name);
                return Err(anyhow!(msg));
            }
        },
    }
    Ok(())
}
