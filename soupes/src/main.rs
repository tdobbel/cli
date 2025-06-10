use anyhow::Result;
use scraper::{Html, Selector};

fn main() -> Result<()> {
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
            let p_selector = Selector::parse("p").unwrap();
            let p = soupe_div.select(&p_selector).next();
            let text = p.unwrap().text().collect::<Vec<_>>().join(" ");
            let soups = text.split("Soupe").collect::<Vec<_>>();
            if soups.len() < 2 {
                println!("Pas de soupe cette semaine 😭");
                return Ok(());
            }
            let is_suggestion = soups[1].trim() == "suggestion";
            if soups.len() == 2 && is_suggestion {
                println!("Pas de soupe cette semaine 😭");
                return Ok(());
            }
            let n_skip = if is_suggestion { 2 } else { 1 };
            println!("Voici les soupes de la semaine 🍲:");
            for soup in soups.iter().skip(n_skip) {
                println!("* Soupe {}", soup.trim());
            }
        }
    }
    Ok(())
}
