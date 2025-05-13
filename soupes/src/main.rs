use regex::Regex;

fn main() {
    let re = Regex::new(r"Soupe de ([^<]+)").unwrap();
    let url = "https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln";
    let resp = reqwest::blocking::get(String::from(url)).expect("Request failed to start url");
    let body = resp.text().expect("Failed to get the response");
    let soups: Vec<&str> = re
        .captures_iter(body.as_str())
        .filter_map(|cap| cap.get(1))
        .map(|cap| cap.as_str())
        .collect();
    if soups.is_empty() {
        println!("Pas de soupe cette semaine 😭");
    } else {
        println!("Voici les soupes de la semaine 🍲:");
        for soup in soups {
            println!("* Soupe de {}", soup);
        }
    }
}
