use anyhow::{anyhow, Result};
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

    #[arg(long, short, default_value_t = false)]
    directory_only: bool,
}

fn display_path(path: &Path) -> Option<ColoredString> {
    let name = path.file_name().unwrap().to_string_lossy();
    if name.starts_with(".") || name == "__pycache__" {
        return None;
    }
    let mut result = if path.is_dir() {
        format!("📁{}", name)
    } else {
        match path.extension() {
            None => format!("{}", name),
            Some(ext) => match ext.to_str().unwrap() {
                "rs" => format!(" {}", name),
                "go" => format!(" {}", name),
                "py" => format!(" {}", name),
                "zig" => format!(" {}", name),
                "c" => format!(" {}", name),
                "cpp" => format!(" {}", name),
                "h" => format!(" {}", name),
                "hpp" => format!(" {}", name),
                "js" => format!(" {}", name),
                "html" => format!(" {}", name),
                "css" => format!(" {}", name),
                "json" => format!(" {}", name),
                "toml" => format!(" {}", name),
                "zip" => format!(" {}", name),
                "tar" => format!(" {}", name),
                "gz" => format!(" {}", name),
                "sh" => format!(" {}", name),
                _ => format!("{}", name),
            },
        }
    };
    if path.is_symlink() {
        result = format!("{} -> {}", result, path.read_link().unwrap().display());
    }
    if path.is_dir() {
        Some(result.green().bold())
    } else {
        Some(result.normal())
    }
}

fn print_tree(
    path: &Path,
    prefix: &str,
    depth: usize,
    max_depth: Option<usize>,
    directory_only: bool,
) -> Result<()> {
    let mut children = path
        .read_dir()?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    children.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for (i, child) in children.iter().enumerate() {
        if directory_only && !child.is_dir() {
            continue;
        }
        let root_char = if i == children.len() - 1 {
            "└"
        } else {
            "├"
        };
        let child_name = if let Some(name) = display_path(child) {
            name
        } else {
            continue;
        };
        println!("{}{}── {}", prefix, root_char, child_name);
        if child.is_dir() && !child.is_symlink() {
            if max_depth.is_some() && depth >= max_depth.unwrap() {
                continue;
            }
            let new_prefix = if i == children.len() - 1 {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            print_tree(child, &new_prefix, depth + 1, max_depth, directory_only)?;
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
    println!("{}", display_path(base_path).unwrap());
    print_tree(base_path, "", 1, args.max_depth, args.directory_only)?;
    Ok(())
}
