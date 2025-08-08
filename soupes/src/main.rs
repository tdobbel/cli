use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDate};
use regex::Regex;
use scraper::{Html, Selector};

pub fn get_month_num(month_name: &str) -> Result<u32> {
    match month_name {
        "janvier" => Ok(1),
        "fevrier" => Ok(2),
        "mars" => Ok(3),
        "avril" => Ok(4),
        "mai" => Ok(5),
        "juin" => Ok(6),
        "juillet" => Ok(7),
        "aout" => Ok(8),
        "septembre" => Ok(9),
        "octobre" => Ok(10),
        "novembre" => Ok(11),
        "décembre" => Ok(12),
        _ => Err(anyhow!("Unknown month: {}", month_name)),
    }
}

fn get_ymd(date: &str) -> (Option<i32>, u32, u32) {
    let parts: Vec<&str> = date.split(' ').collect();
    let day = parts[0].parse::<u32>().unwrap();
    let month = get_month_num(parts[1]).unwrap();
    let year = if parts.len() > 2 {
        Some(parts[2].parse::<i32>().unwrap())
    } else {
        None
    };
    (year, month, day)
}

fn isclosed(body: &str) -> bool {
    let pattern = Regex::new(r"fermée du ([0-9 A-zÀ-ÿ]+) au ([0-9 A-zÀ-ÿ]+) (\d+)").unwrap();
    let now = Local::now().date_naive();
    for cap in pattern.captures_iter(body) {
        let start_date_str = cap.get(1).unwrap().as_str();
        let end_day_month = cap.get(2).unwrap().as_str();
        let end_year_str = cap.get(3).unwrap().as_str();
        let end_date_str = format!("{} {}", end_day_month, end_year_str);
        let (start_year, start_month, start_day) = get_ymd(start_date_str);
        let (end_year, end_month, end_day) = get_ymd(&end_date_str);
        let year = end_year.unwrap();
        let start_date = NaiveDate::from_ymd_opt(
            if let Some(y) = start_year { y } else { year },
            start_month,
            start_day,
        )
        .unwrap();
        let end_date = NaiveDate::from_ymd_opt(year, end_month, end_day).unwrap();
        if (now >= start_date) && (now <= end_date) {
            return true;
        }
    }
    false
}

fn main() -> Result<()> {
    let url = "https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln";
    let resp = reqwest::blocking::get(String::from(url))?;
    let body = resp.text()?;
    if isclosed(&body) {
        println!("Pas de soupe cette semaine 😭");
        return Ok(());
    }
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
                println!("* Soupe {}", soup.trim())
            }
        }
    }
    Ok(())
}
