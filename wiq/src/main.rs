use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::env;
use std::process::Command;

struct User {
    running: u32,
    pending: u32,
    partitions: HashSet<String>,
}

impl User {
    fn new() -> User {
        User {
            running: 0,
            pending: 0,
            partitions: HashSet::new(),
        }
    }

    fn cmp(&self, other: &User) -> Ordering {
        let total_a = self.running + self.pending;
        let total_b = other.running + other.pending;
        total_b.cmp(&total_a)
    }
}

fn main() -> Result<()> {
    let re = Regex::new(r"\[(\d+)-([0-9\%]+)\]").unwrap();
    let mut args = vec!["--noheader".to_string(), "-o %.20u %t %P %i".to_string()];
    let message_end = match env::args().nth(1) {
        Some(v) => {
            args.push(format!("--partition={}", v));
            format!("partition {}", v)
        }
        None => "the queue".to_string(),
    };
    let output = Command::new("squeue")
        .args(args)
        .output()
        .expect("failed to execute process");
    let queue = String::from_utf8_lossy(&output.stdout).to_string();
    let n_lines = queue.lines().count();
    if n_lines == 0 {
        println!("🥳🎉 There are no jobs in {} 🎉🥳", message_end);
        return Ok(());
    }
    let mut counter = HashMap::<String, User>::new();
    for line in queue.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (usr_name, status, partitions_) = (words[0], words[1], words[2]);
        let partitions: Vec<&str> = partitions_.split(',').collect();
        let user = counter.entry(usr_name.to_string()).or_insert(User::new());
        for par in partitions.iter() {
            user.partitions.insert(par.to_string());
        }
        if status == "R" {
            user.running += 1;
        } else if status == "PD" {
            let jobid = words[3];
            if let Some(caps) = re.captures(jobid) {
                let start = caps[1].parse::<u32>()?;
                let end = (if let Some(n) = caps[2].find('%') {
                    &caps[2][..n]
                } else {
                    &caps[2]
                })
                .parse::<u32>()?;
                user.pending += end - start + 1;
            } else {
                user.pending += 1;
            }
        }
    }
    let n_job = counter.values().map(|x| x.running + x.pending).sum::<u32>();
    let mut users: Vec<String> = counter.keys().map(|x| x.to_string()).collect();
    users.sort_by(|a, b| counter[a].cmp(&counter[b]));
    println!(
        "There are {} jobs in {}:",
        n_job.to_string().bold(),
        message_end
    );
    for usr_name in users.iter() {
        let user = &counter[usr_name];
        let mut parts: Vec<&str> = user.partitions.iter().map(|p| p.as_str()).collect();
        parts.sort();
        println!(
            "-> {:<12}: {:>4} running, {:>4} pending  ({})",
            usr_name.blue(),
            user.running.to_string().green().bold(),
            user.pending.to_string().yellow().bold(),
            parts.join(", ").cyan()
        );
    }

    Ok(())
}
