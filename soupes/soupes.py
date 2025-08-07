#!/usr/bin/env python3

from datetime import datetime
import requests
import re
from bs4 import BeautifulSoup

now = datetime.now()
pattern = re.compile("fermée du (\d+) ([A-zÀ-ÿ]+) au (\d+) ([A-zÀ-ÿ]+) (\d+)")

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


def main() -> None:
    resp = requests.get("https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln")
    for match in pattern.finditer(resp.text):
        start_day, start_month, end_day, end_month, year = match.groups()
        start_date = datetime(int(year), month_num[start_month.lower()], int(start_day))
        end_date = datetime(int(year), month_num[end_month.lower()], int(end_day))
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
