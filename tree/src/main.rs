use anyhow::{Result, anyhow};
use clap::Parser;
use colored::{ColoredString, Colorize};
use std::env;
use std::io;
use std::path::Path;

#[derive(Parser, Debug)]
struct Arguments {
    #[arg(default_value = ".")]
    directory: String,

    #[arg(long)]
    max_depth: Option<usize>,
}

fn display_path(path: &Path) -> ColoredString {
    let name = path.file_name().unwrap().to_string_lossy();
    if path.is_dir() {
        format!("📁{}", name).green().bold()
    } else {
        name.normal()
    }
}

fn print_tree(path: &Path, prefix: &str, depth: usize, max_depth: Option<usize>) -> Result<()> {
    let children = path
        .read_dir()?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    for (i, child) in children.iter().enumerate() {
        let root_char = if i == children.len() - 1 {
            "└"
        } else {
            "├"
        };
        println!("{}{}── {}", prefix, root_char, display_path(child));
        if child.is_dir() {
            if max_depth.is_some() && depth >= max_depth.unwrap() {
                continue;
            }
            let new_prefix = if i == children.len() - 1 {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            print_tree(child, &new_prefix, depth + 1, max_depth)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Arguments::parse();
    let cwd = env::current_dir()?;
    let base_path = if args.directory == *"." {
        Path::new(&cwd)
    } else {
        Path::new(&args.directory)
    };
    if !base_path.exists() {
        return Err(anyhow!("{} does not exist", base_path.display()));
    }
    if base_path.is_file() {
        return Err(anyhow!("{} is a file", base_path.display()));
    }
    println!("{}", display_path(base_path));
    print_tree(base_path, "", 1, args.max_depth)?;
    Ok(())
}
