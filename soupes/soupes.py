#!/usr/bin/env python3

from datetime import datetime
import requests
import re
from bs4 import BeautifulSoup

now = datetime.now()
pattern = re.compile("fermée du ([0-9 A-zÀ-ÿ]+) au ([0-9 A-zÀ-ÿ]+) (\d+)")

month_num = {
    "janvier": 1,
    "février": 2,
    "mars": 3,
    "avril": 4,
    "mai": 5,
    "juin": 6,
    "juillet": 7,
    "août": 8,
    "septembre": 9,
    "octobre": 10,
    "novembre": 11,
    "décembre": 12,
}


def get_date_parts(date_str: str) -> tuple[int | None, int, int]:
    date_parts = date_str.split()
    day =  int(date_parts[0]) 
    month = month_num[date_parts[1].lower()]
    year = None
    if len(date_parts) > 2:
        year = int(date_parts[2])
    return year, month, day


def main() -> None:
    resp = requests.get("https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln")
    for match in pattern.finditer(resp.text):
        start_date, end_date, year = match.groups()
        start_date_parts = get_date_parts(start_date)
        end_date_parts = get_date_parts(f"{end_date} {year}")
        if start_date_parts[0] is None:
            start_date_parts = (end_date_parts[0], *start_date_parts[1:])
        start_date = datetime(*start_date_parts)
        end_date = datetime(*end_date_parts)
        if start_date <= now <= end_date:
            print("Pas de soupe cette semaine 😭")
            return
    scrapper = BeautifulSoup(resp.text, "html.parser")
    div = scrapper.find("div", attrs={"id": "Nos soupes"})
    p = div.find("p")
    soups = p.text.strip().split("Soupe")
    if len(soups) < 2:
        print("Pas de soupe cette semaine 😭")
        return
    if len(soups) == 2 and soups[1].strip() == "suggestion":
        print("Pas de soupe cette semaine 😭")
        return
    print("Voici les soupes de la semaine 🍲:")
    for soup in soups[1:]:
        print(f"* Soupe {soup.strip()}")


if __name__ == "__main__":
    main()
