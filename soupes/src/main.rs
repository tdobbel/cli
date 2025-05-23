use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};

fn main() -> Result<()> {
    let re = Regex::new(r"Soupe ([^<]+)").unwrap();
    let url = "https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln";
    let resp = reqwest::blocking::get(String::from(url))?;
    let body = resp.text()?;
    let document = Html::parse_document(&body);
    let div_selector = Selector::parse("div").unwrap();
    let soup_div = document
        .select(&div_selector)
        .find(|div| div.value().attr("id") == Some("Nos soupes"));
    match soup_div {
        None => return Err(anyhow::anyhow!("Où sont les soupes ?! 😭")),
        Some(soupe_div) => {
            let text = soupe_div.html();
            let soups: Vec<&str> = re
                .captures_iter(text.as_str())
                .filter_map(|cap| cap.get(1))
                .map(|cap| cap.as_str())
                .collect();
            if soups.is_empty() {
                println!("Pas de soupe cette semaine 😭");
            } else {
                println!("Voici les soupes de la semaine 🍲:");
                for soup in soups {
                    println!("* Soupe {}", soup);
                }
            }
        }
    }
    Ok(())
}
